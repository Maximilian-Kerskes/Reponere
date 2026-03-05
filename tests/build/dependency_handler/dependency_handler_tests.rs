use std::cell::RefCell;
use std::collections::HashMap;

use reponere::build::package_manager::manager::{PackageManagerApi, PackageManagerError};

pub struct MockPackageManager {
    installed: RefCell<HashMap<String, String>>,
    available: RefCell<HashMap<String, String>>,
}

impl MockPackageManager {
    pub fn new() -> Self {
        Self {
            installed: RefCell::new(HashMap::new()),
            available: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_installed(self, name: &str, version: &str) -> Self {
        self.installed
            .borrow_mut()
            .insert(name.to_string(), version.to_string());
        self // don't overwrite available
    }

    pub fn with_available(self, name: &str, version: &str) -> Self {
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

    fn reverse_dependencies(&self, package: &str) -> Result<Vec<String>, PackageManagerError> {
        Ok(Vec::new())
    }
}

mod tests {
    use super::*;
    use reponere::build::{
        dependency_handler::{
            build_dependency_guard::BuildDependencyGuard,
            dependency_handler::{DependencyError, DependencyHandler},
        },
        package::package::{Dependencies, Dependency},
        package_manager::manager::PackageManagerApi,
    };

    fn make_dependency(name: &str, version_req: Option<&str>) -> Dependency {
        Dependency {
            name: name.to_string(),
            version_req: version_req.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_install_runtime_dependencies_when_not_installed() {
        let mock_pm = MockPackageManager::new().with_available("foo", "1.0.0");
        let deps = Dependencies {
            runtime: vec![make_dependency("foo", None)],
            build: vec![],
        };
        let handler = DependencyHandler::new(&mock_pm, deps);

        let mut errors = Vec::new();
        handler.install_runtime_dependencies(&mut errors);

        assert!(errors.is_empty());
        // install method inserts "foo", but get_installed_version expects "foo@version"
        let installed_version = mock_pm.get_installed_version("foo").unwrap();
        assert_eq!(installed_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_install_build_dependencies_with_version_check() {
        let mock_pm = MockPackageManager::new()
            .with_available("bar", "2.3.4")
            .with_installed("bar", "2.0.0"); // older version
        let deps = Dependencies {
            runtime: vec![],
            build: vec![make_dependency("bar", Some(">=2.1.0"))],
        };
        let handler = DependencyHandler::new(&mock_pm, deps);

        let mut errors = Vec::new();
        let installed = handler.install_build_dependencies(&mut errors);

        assert_eq!(installed, vec!["bar".to_string()]);
        assert!(errors.is_empty());

        let installed_version = mock_pm.get_installed_version("bar").unwrap();
        assert_eq!(installed_version, Some("2.3.4".to_string()));
    }

    #[test]
    fn test_dependency_not_available_triggers_error() {
        let mock_pm = MockPackageManager::new(); // no available versions
        let deps = Dependencies {
            runtime: vec![make_dependency("baz", Some(">=1.0.0"))],
            build: vec![],
        };
        let handler = DependencyHandler::new(&mock_pm, deps);

        let mut errors = Vec::new();
        handler.install_runtime_dependencies(&mut errors);

        assert_eq!(errors.len(), 1);
        match &errors[0] {
            DependencyError::AvailableVersionCheckFailed { dependency, .. } => {
                assert_eq!(dependency, "baz");
            }
            _ => panic!("Expected AvailableVersionCheckFailed"),
        }
    }

    #[test]
    fn test_dependency_already_installed_satisfying_version() {
        let mock_pm = MockPackageManager::new().with_installed("foo", "1.2.3");
        let deps = Dependencies {
            runtime: vec![make_dependency("foo", Some(">=1.0.0"))],
            build: vec![],
        };
        let handler = DependencyHandler::new(&mock_pm, deps);

        let mut errors = Vec::new();
        handler.install_runtime_dependencies(&mut errors);

        assert!(errors.is_empty());
        let installed_version = mock_pm.get_installed_version("foo").unwrap();
        assert_eq!(installed_version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_build_dependency_guard_uninstalls_on_drop() {
        let mock_pm = MockPackageManager::new().with_installed("temp", "0.1.0");

        let guard = BuildDependencyGuard {
            package_manager: &mock_pm,
            installed: vec!["temp@0.1.0".to_string()],
        };
        drop(guard);

        let installed_version = mock_pm.get_installed_version("temp@0.1.0").unwrap();
        assert!(installed_version.is_none());
    }
}
