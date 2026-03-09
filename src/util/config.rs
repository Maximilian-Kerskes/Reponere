use serde::{Deserialize, Serialize};

use dirs;
use std::{fs, path::PathBuf};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub index_path: PathBuf,
    pub registry_path: PathBuf,
    pub packages_path: PathBuf,
}

impl Config {
    fn create_default() -> Result<String, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().expect("Unable to get home directory");
        let config = Config {
            index_path: home.join(".config/reponere/index.json"),
            registry_path: home.join(".config/reponere/registry"),
            packages_path: home.join(".config/reponere/packages.json"),
        };
        let serialized = toml::to_string(&config)?;
        std::fs::write(
            home.join(".config/reponere/config.toml"),
            serialized.clone(),
        )?;
        Ok(serialized.clone())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().expect("Unable to get home directory");
        let config_path = home.join(".config/reponere/config.toml");
        let serialized = fs::read_to_string(&config_path).or_else(|_| Config::create_default())?;
        let config: Config = toml::from_str(&serialized)?;
        Ok(config)
    }
}
