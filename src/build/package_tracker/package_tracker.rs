use crate::build::package::package::InstalledPackage;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageTrackerError {
    #[error("Error handling json: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to write to file: {0}")]
    WriteError(#[from] std::io::Error),
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct PackageTracker {
    packages: HashMap<String, InstalledPackage>,
}

impl PackageTracker {
    pub fn load(path: &str) -> Result<Self, PackageTrackerError> {
        match fs::read_to_string(path) {
            Ok(data) => Ok(serde_json::from_str(&data)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(PackageTracker::default()),
            Err(e) => Err(e.into()), // other IO errors propagated
        }
    }

    pub fn save(&self, path: &str) -> Result<(), PackageTrackerError> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn add_package(&mut self, pkg: InstalledPackage) {
        self.packages.insert(pkg.name.clone(), pkg);
    }

    pub fn get_package(&self, name: &str) -> Option<&InstalledPackage> {
        self.packages.get(name)
    }

    pub fn get_packages(&self) -> &HashMap<String, InstalledPackage> {
        &self.packages
    }
}
