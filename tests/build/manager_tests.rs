mod tests {
    use reponere::build::manager::{PackageManager, PackageManagerApi};

    #[test]
    #[ignore = "requires sudo and modifies system"]
    pub fn test_install_and_uninstall() {
        let manager = PackageManager::get_package_manager(true).unwrap();

        let result = manager.install("minicom");
        assert!(
            result.is_ok(),
            "Expected install to succeed, got {result:?}",
        );

        let result = manager.install("asfd");
        assert!(result.is_err(), "Expected install to fail, got {result:?}",);

        let result = manager.uninstall("minicom");
        assert!(
            result.is_ok(),
            "Expected uninstall to succeed, got {result:?}"
        );

        let result = manager.uninstall("asfd");
        assert!(
            result.is_err(),
            "Expected uninstall to fail, got {result:?}",
        );
    }

    #[test]
    #[ignore = "requires sudo and modifies system"]
    pub fn test_get_installed_version() {
        let manager = PackageManager::get_package_manager(true).unwrap();
        let result = manager.get_installed_version("cmake");
        assert!(
            result.is_ok(),
            "Expected get_installed_version to succeed, got {result:?}"
        );
        let result_option = result.unwrap();
        assert!(
            result_option.is_some(),
            "Expected get_installed_version to return Some, got {result_option:?}"
        );

        let result = manager.get_installed_version("asfd");
        assert!(
            result.is_ok(),
            "Expected get_installed_version to succeed, got {result:?}"
        );
        let result_option = result.unwrap();
        assert!(
            result_option.is_none(),
            "Expected get_installed_version to return None, got {result_option:?}"
        );
    }

    #[test]
    #[ignore = "requires sudo and modifies system"]
    pub fn test_get_available_version() {
        let manager = PackageManager::get_package_manager(true).unwrap();
        let result = manager.get_available_version("gnome-menus");
        assert!(
            result.is_ok(),
            "Expected get_installed_version to succeed, got {result:?}"
        );
        let result_option = result.unwrap();
        assert!(
            result_option.is_some(),
            "Expected get_installed_version to return Some, got {result_option:?}"
        );

        let result = manager.get_available_version("asfd");
        assert!(
            result.is_ok(),
            "Expected get_installed_version to succeed, got {result:?}"
        );
        let result_option = result.unwrap();
        assert!(
            result_option.is_none(),
            "Expected get_installed_version to return None, got {result_option:?}"
        );
    }
}
