use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::build::manager::{PackageManagerApi, PackageManagerError};

pub struct MockPackageManager {
    installed: RefCell<HashMap<String, String>>, // store name -> version
    available: RefCell<HashMap<String, String>>,
}

impl MockPackageManager {
    pub fn new() -> Self {
        Self {
            installed: RefCell::new(HashMap::new()),
            available: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_installed(mut self, name: &str, version: &str) -> Self {
        self.installed
            .borrow_mut()
            .insert(name.to_string(), version.to_string());
        self // don't overwrite available
    }

    pub fn with_available(mut self, name: &str, version: &str) -> Self {
        self.available
            .borrow_mut()
            .insert(name.to_string(), version.to_string());
        self
    }
}

impl PackageManagerApi for MockPackageManager {
    fn install(&self, package: &str) -> Result<(), PackageManagerError> {
        if let Some(version) = self.available.borrow().get(package) {
            self.installed
                .borrow_mut()
                .insert(package.to_string(), version.clone());
        }
        Ok(())
    }

    fn uninstall(&self, package: &str) -> Result<(), PackageManagerError> {
        self.installed.borrow_mut().remove(package);
        Ok(())
    }

    fn get_installed_version(&self, package: &str) -> Result<Option<String>, PackageManagerError> {
        Ok(self.installed.borrow().get(package).cloned())
    }

    fn get_available_version(&self, package: &str) -> Result<Option<String>, PackageManagerError> {
        Ok(self.available.borrow().get(package).cloned())
    }
}
