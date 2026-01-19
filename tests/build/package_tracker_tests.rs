mod tests {
    use reponere::build::package::{Dependency, InstalledPackage};
    use reponere::build::package_tracker::PackageTracker;
    use tempfile::NamedTempFile;

    fn dummy_dependency() -> Dependency {
        Dependency {
            name: "serde".to_string(),
            version_req: Some(">=1.0".to_string()),
        }
    }

    fn dummy_package(name: &str) -> InstalledPackage {
        InstalledPackage {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            install_path: "/usr/local".to_string(),
            dependencies: vec![dummy_dependency()],
        }
    }

    #[test]
    fn test_load_nonexistent_file_returns_default() {
        let tracker = PackageTracker::load("nonexistent.json").unwrap();
        assert!(tracker.get_packages().is_empty());
    }

    #[test]
    fn test_add_package() {
        let mut tracker = PackageTracker::default();
        let pkg = dummy_package("mypkg");

        tracker.add_package(pkg);

        assert_eq!(tracker.get_packages().len(), 1);
        let stored = tracker.get_package("mypkg").unwrap();
        assert_eq!(stored.name, "mypkg");
        assert_eq!(stored.version, "1.0.0");
        assert_eq!(stored.install_path, "/usr/local");
        assert_eq!(stored.dependencies.len(), 1);
    }

    #[test]
    fn test_save_and_load() {
        let mut tracker = PackageTracker::default();
        let pkg = dummy_package("mypkg");
        tracker.add_package(pkg);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Save
        tracker.save(path).unwrap();

        // Load back
        let loaded = PackageTracker::load(path).unwrap();
        assert_eq!(loaded.get_packages().len(), 1);
        assert!(loaded.get_package("mypkg").is_some());
    }

    #[test]
    fn test_multiple_packages() {
        let mut tracker = PackageTracker::default();
        tracker.add_package(dummy_package("pkg1"));
        tracker.add_package(dummy_package("pkg2"));

        assert_eq!(tracker.get_packages().len(), 2);
        assert!(tracker.get_package("pkg1").is_some());
        assert!(tracker.get_package("pkg2").is_some());
    }
}
