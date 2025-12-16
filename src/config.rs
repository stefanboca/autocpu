use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use eyre::bail;

use crate::Preset;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub upower_battery_path: Option<String>,
    pub on_battery: String,
    pub on_wallpower: String,
    pub presets: HashMap<String, Preset>,
}

impl Config {
    pub fn load(path: &Path) -> eyre::Result<Arc<Self>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;

        if !config.presets.contains_key(&config.on_battery) {
            bail!("config: `on_battery` is set to a preset that does not exist.");
        }
        if !config.presets.contains_key(&config.on_wallpower) {
            bail!("config: `on_wallpower` is set to a preset that does not exist.");
        }

        Ok(Arc::new(config))
    }
}
