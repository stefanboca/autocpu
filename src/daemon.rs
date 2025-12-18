use std::collections::HashMap;
use std::sync::Arc;

use futures::stream::StreamExt;
use tokio::sync::{Mutex, mpsc};
use upower_dbus::{BatteryType, DeviceProxy, UPowerProxy};

use crate::{Config, PowerState};

struct Daemon {
    config: Arc<Config>,
    preset_tx: mpsc::Sender<String>,
    current_state: Arc<Mutex<Option<PowerState>>>,
    current_presets: Arc<Mutex<HashMap<PowerState, String>>>,
}

impl Daemon {
    pub fn new(
        config: Arc<Config>,
        preset_tx: mpsc::Sender<String>,
        current_state: Arc<Mutex<Option<PowerState>>>,
        current_presets: Arc<Mutex<HashMap<PowerState, String>>>,
    ) -> Self {
        Self {
            config,
            preset_tx,
            current_state,
            current_presets,
        }
    }
}

#[zbus::interface(name = "org.stefanboca.AutoCpu")]
impl Daemon {
    #[zbus(property(emits_changed_signal = "const"))]
    async fn available_presets(&self) -> Vec<&str> {
        self.config.presets.keys().map(|s| s.as_ref()).collect()
    }

    #[zbus(property(emits_changed_signal = "true"))]
    async fn current_preset(&self) -> String {
        let Some(state) = *self.current_state.lock().await else {
            return String::new();
        };
        self.current_presets
            .lock()
            .await
            .get(&state)
            .cloned()
            .unwrap_or_else(String::new)
    }

    #[zbus(property)]
    async fn set_current_preset(&mut self, preset_name: &str) -> zbus::fdo::Result<()> {
        if self.config.presets.contains_key(preset_name) {
            self.preset_tx
                .send(preset_name.to_string())
                .await
                .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))
        } else {
            Err(zbus::fdo::Error::Failed(
                "profile name is not valid; check available presets.".to_string(),
            ))
        }
    }
}

async fn worker(
    config: Arc<Config>,
    conn: zbus::Connection,
    mut preset_rx: mpsc::Receiver<String>,
    current_state: Arc<Mutex<Option<PowerState>>>,
    current_presets: Arc<Mutex<HashMap<PowerState, String>>>,
) -> eyre::Result<()> {
    let upower = UPowerProxy::new(&conn).await?;
    let device_paths = upower.enumerate_devices().await?;

    let device: Option<DeviceProxy<'_>> = {
        let mut device = None;
        for device_path in device_paths {
            let matches_config = config
                .upower_battery_path
                .as_ref()
                .is_some_and(|battery| battery == device_path.as_str());

            let dev = DeviceProxy::new(&conn, device_path.clone()).await?;

            if matches_config || dev.type_().await? == BatteryType::Battery {
                log::info!("Found battery `{device_path}`");
                device = Some(dev);
                break;
            }
        }
        device
    };

    let mut stream = if let Some(device) = device {
        device
            .receive_state_changed()
            .await
            .then(|event| async move {
                eyre::Result::<PowerState>::Ok(PowerState::from(event.get().await?))
            })
            .boxed()
    } else {
        log::info!("No battery found, assuming wallpower");
        futures::stream::once(async { Ok(PowerState::OnWallpower) }).boxed()
    };

    loop {
        let preset_name = tokio::select! {
            biased;

            Some(preset_name) = preset_rx.recv() => {
                {
                    if let Some(current_state) = *current_state.lock().await {
                        let mut current_presets_ = current_presets.lock().await;
                        if current_presets_.get(&current_state).is_some_and(|preset| *preset == preset_name) {
                            continue;
                        };
                        current_presets_.insert(current_state, preset_name.clone());
                    }
                }

                preset_name
            }
            Some(state) = stream.next() => {
                let state = state?;
                *current_state.lock().await = Some(state);

                current_presets.lock().await.get(&state).unwrap().clone()
            }

            else => {
                log::debug!("Worker exiting");
                break;
            }
        };

        if let Some(preset) = config.presets.get(&preset_name) {
            log::info!("Applying preset `{preset_name}`");
            preset.apply();
        };
    }

    Ok(())
}

pub async fn run(config: Arc<Config>) -> eyre::Result<()> {
    log::info!("Starting...");
    let (preset_tx, preset_rx) = mpsc::channel(1);
    let current_state = Arc::new(Mutex::new(None));
    let current_presets = Arc::new(Mutex::new(HashMap::from([
        (PowerState::OnBattery, config.on_battery.clone()),
        (PowerState::OnWallpower, config.on_wallpower.clone()),
    ])));

    let daemon = Daemon::new(
        config.clone(),
        preset_tx,
        current_state.clone(),
        current_presets.clone(),
    );

    let conn = zbus::connection::Builder::system()?
        .name("org.stefanboca.AutoCpu")?
        .serve_at("/org/stefanboca/AutoCpu", daemon)?
        .build()
        .await?;

    let handle = tokio::spawn(worker(
        config,
        conn,
        preset_rx,
        current_state,
        current_presets,
    ));
    log::info!("Started");

    handle.await?
}
