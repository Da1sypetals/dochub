use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const DEFAULT_SANE_SIZE_MB: u64 = 16;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "sane-size")]
    pub sane_size: Option<u64>,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(rename = "skill-dir", default)]
    pub skill_dir: Vec<String>,
    #[serde(default)]
    pub hub: BTreeMap<String, String>,
}

impl Config {
    pub fn sane_size_bytes(&self) -> u64 {
        self.sane_size.unwrap_or(DEFAULT_SANE_SIZE_MB) * 1024 * 1024
    }
}

pub fn config_path() -> Result<PathBuf, String> {
    let home =
        dirs::home_dir().ok_or_else(|| "Could not resolve the home directory.".to_string())?;
    Ok(home.join(".dochub").join("hub.toml"))
}

pub fn load() -> Result<Config, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read {}: {err}", path.display()))?;

    toml::from_str(&contents).map_err(|err| format!("Failed to parse {}: {err}", path.display()))
}

pub fn save(config: &Config) -> Result<PathBuf, String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create {}: {err}", parent.display()))?;
    }

    let contents =
        toml::to_string(config).map_err(|err| format!("Failed to serialize config: {err}"))?;

    fs::write(&path, contents)
        .map_err(|err| format!("Failed to write {}: {err}", path.display()))?;
    Ok(path)
}
