mod tests {
    use reponere::{
        build::package_manager::manager::PackageManagerApi, mock::mock_manager::MockPackageManager,
    };

    #[test]
    fn test_install_package() {
        let manager = MockPackageManager::new().with_available("foo", "1.0.0");
        assert!(manager.get_installed_version("foo").unwrap().is_none());

        manager.install("foo").unwrap();
        let version = manager.get_installed_version("foo").unwrap();
        assert_eq!(version, Some("1.0.0".to_string())); // returns available version
    }

    #[test]
    fn test_uninstall_package() {
        let manager = MockPackageManager::new()
            .with_available("bar", "2.3.4")
            .with_installed("bar", "2.3.4");

        assert_eq!(
            manager.get_installed_version("bar").unwrap(),
            Some("2.3.4".to_string())
        );

        manager.uninstall("bar").unwrap();
        assert!(manager.get_installed_version("bar").unwrap().is_none());
    }

    #[test]
    fn test_get_available_version() {
        let manager = MockPackageManager::new().with_available("baz", "0.9.1");

        let version = manager.get_available_version("baz").unwrap();
        assert_eq!(version, Some("0.9.1".to_string()));

        let missing_version = manager.get_available_version("nonexistent").unwrap();
        assert!(missing_version.is_none());
    }

    #[test]
    fn test_with_installed_and_available_chain() {
        let manager = MockPackageManager::new()
            .with_installed("foo", "1.2.3")
            .with_available("bar", "4.5.6");

        // Installed package returns available version
        assert_eq!(
            manager.get_installed_version("foo").unwrap(),
            Some("1.2.3".to_string())
        );

        // Check available
        assert_eq!(
            manager.get_available_version("bar").unwrap(),
            Some("4.5.6".to_string())
        );
    }

    #[test]
    fn test_install_then_uninstall() {
        let manager = MockPackageManager::new().with_available("qux", "5.6.7");

        manager.install("qux").unwrap();
        assert_eq!(
            manager.get_installed_version("qux").unwrap(),
            Some("5.6.7".to_string())
        );

        manager.uninstall("qux").unwrap();
        assert!(manager.get_installed_version("qux").unwrap().is_none());
    }
}
