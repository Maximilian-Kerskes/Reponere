mod tests {
    use reponere::{
        build::{
            dependency_handler::build_dependency_guard::BuildDependencyGuard,
            package_manager::manager::PackageManagerApi,
        },
        mock::mock_manager::MockPackageManager,
    };

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
