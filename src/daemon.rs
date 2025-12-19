use std::collections::HashMap;
use std::sync::Arc;

use futures::stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use upower_dbus::{BatteryType, DeviceProxy, UPowerProxy};

use crate::{Config, PowerState};

enum Message {
    GetPreset(oneshot::Sender<String>),
    SetPreset(String),
}

struct Daemon {
    config: Arc<Config>,
    message_tx: mpsc::Sender<Message>,
}

impl Daemon {
    pub fn new(config: Arc<Config>, message_tx: mpsc::Sender<Message>) -> Self {
        Self { config, message_tx }
    }
}

#[zbus::interface(name = "org.stefanboca.AutoCpu", proxy)]
impl Daemon {
    #[zbus(property(emits_changed_signal = "const"))]
    async fn available_presets(&self) -> Vec<String> {
        log::trace!("Daemon::available_presets");
        self.config.presets.keys().cloned().collect()
    }

    #[zbus(property(emits_changed_signal = "true"))]
    async fn current_preset(&self) -> zbus::fdo::Result<String> {
        log::trace!("Daemon::current_preset");
        let (tx, rx) = oneshot::channel();
        self.message_tx
            .send(Message::GetPreset(tx))
            .await
            .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))?;
        rx.await
            .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))
    }

    #[zbus(property)]
    async fn set_current_preset(&mut self, preset_name: &str) -> zbus::fdo::Result<()> {
        log::trace!("Daemon::set_current_preset({preset_name:?})");
        self.message_tx
            .send(Message::SetPreset(preset_name.to_string()))
            .await
            .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))
    }
}

async fn worker(
    config: Arc<Config>,
    conn: zbus::Connection,
    mut message_rx: mpsc::Receiver<Message>,
) -> eyre::Result<()> {
    // First, attempt to use the battery specified in the config
    let battery = if let Some(upower_battery_path) = config.upower_battery_path.as_deref()
        && let Ok(battery) = DeviceProxy::new(&conn, upower_battery_path)
            .await
            .inspect_err(|err| {
                log::warn!("Failed to create UPower DeviceProxy for `{upower_battery_path}`: {err}")
            }) {
        Some(battery)
    }
    // If there is no configured battery or the configured battery is not found, try to find another battery
    else if let Ok(upower) = UPowerProxy::new(&conn)
        .await
        .inspect_err(|err| log::warn!("Failed to create UPowerProxy: {err}"))
        && let Ok(device_paths) = upower
            .enumerate_devices()
            .await
            .inspect_err(|err| log::debug!("Failed to enumerate UPower devices: {err}"))
    {
        let mut battery = None;
        for device_path in device_paths {
            if let Ok(device) = DeviceProxy::new(&conn, device_path.clone())
                .await
                .inspect_err(|err| {
                    log::debug!("Failed to create UPower DeviceProxy for `{device_path}`: {err}")
                })
                && let Ok(BatteryType::Battery) = device
                    .type_()
                    .await
                    .inspect_err(|err| log::debug!("Failed to get type of `{device_path}`: {err}"))
            {
                battery = Some(device);
                break;
            };
        }
        battery
    } else {
        None
    };

    let mut stream = if let Some(battery) = battery {
        log::info!("Found battery `{}`", battery.inner().path());
        battery
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

    let mut current_state = None;
    let mut current_presets = HashMap::from([
        (PowerState::OnBattery, config.on_battery.clone()),
        (PowerState::OnWallpower, config.on_wallpower.clone()),
    ]);

    loop {
        let preset_name = tokio::select! {
            biased;

            Some(message) = message_rx.recv() => {
                match message {
                    Message::GetPreset(tx) => {
                        let preset_name = if let Some(state) = current_state {
                            current_presets.get(&state).unwrap().clone()
                        } else {
                            String::new()
                        };
                        if let Err(err) = tx.send(preset_name) {
                            log::warn!("Failed to send: {err}")
                        };
                        continue;
                    },
                    Message::SetPreset(preset_name) => {
                        if let Some(current_state) = current_state {
                            if current_presets.get(&current_state).is_some_and(|preset| *preset == preset_name) {
                                continue;
                            };
                            current_presets.insert(current_state, preset_name.clone());
                        }
                        preset_name
                    },
                }
            }
            Some(state) = stream.next() => {
                let state = state?;
                if current_state == Some(state) {
                    continue;
                }
                current_state = Some(state);
                current_presets.get(&state).unwrap().clone()
            }

            else => eyre::bail!("worker exiting")
        };

        if let Some(preset) = config.presets.get(&preset_name) {
            log::info!("Applying preset `{preset_name}`");
            preset.apply();
        };
    }
}

pub async fn run(config: Arc<Config>) -> eyre::Result<()> {
    log::info!("Starting...");
    let (message_tx, message_rx) = mpsc::channel(1);

    let daemon = Daemon::new(config.clone(), message_tx);

    let conn = zbus::connection::Builder::system()?
        .name("org.stefanboca.AutoCpu")?
        .serve_at("/org/stefanboca/AutoCpu", daemon)?
        .build()
        .await?;

    let handle = tokio::spawn(worker(config, conn, message_rx));
    log::info!("Started");

    handle.await?
}
