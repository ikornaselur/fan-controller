use anyhow::Result;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = ".fan-controller.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub target_temp: f32,
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
}

/// Returns the config file path next to the current binary.
pub fn config_path() -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().unwrap_or(Path::new("/"));
    Ok(dir.join(CONFIG_FILENAME))
}

/// Load config from disk. Returns None if the file doesn't exist.
pub fn load() -> Result<Option<Config>> {
    let path = config_path()?;
    if !path.exists() {
        debug!("No config file at {}", path.display());
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)?;
    let config: Config = serde_json::from_str(&contents)?;
    info!("Loaded config from {}", path.display());
    Ok(Some(config))
}

/// Save config to disk.
pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    let contents = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, contents)?;
    debug!("Saved config to {}", path.display());
    Ok(())
}
