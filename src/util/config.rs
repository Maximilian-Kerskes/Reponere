use serde::Deserialize;

use dirs;
use std::path::PathBuf;
use toml;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub index_path: PathBuf,
    pub registry_path: PathBuf,
    pub packages_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().expect("Unable to get home directory");
        let contents = std::fs::read_to_string(home.join(".config/reponere/config.toml"))?;
        Ok(toml::from_str(&contents)?)
    }
}
