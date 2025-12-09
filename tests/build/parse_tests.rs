mod tests {
    use reponere::build::{package::Source, parse::PackageParser};

    #[test]
    pub fn test_parse() {
        let parser = PackageParser::new("./resources/package_build.yaml");
        let package = parser.parse().unwrap();
        assert_eq!(package.name, "mypackage");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.description, Some("package description".to_string()));
        assert_eq!(
            package.source,
            Source::Git {
                repo: "https://github.com/user/mypackage.git".to_string(),
                tag: Some("v1.0.0".to_string()),
                branch: Some("main".to_string()),
                commit: None
            }
        );
        assert_eq!(package.dependencies.runtime.len(), 2);
        assert_eq!(package.dependencies.runtime[0].name, "serde");
        assert_eq!(
            package.dependencies.runtime[0].version_req,
            Some(">=1.0".to_string())
        );
        assert_eq!(package.dependencies.runtime[1].name, "tokio");
        assert_eq!(
            package.dependencies.runtime[1].version_req,
            Some("1.25".to_string())
        );
        assert_eq!(package.dependencies.build.len(), 2);
        assert_eq!(package.dependencies.build[0].name, "cmake");
        assert_eq!(package.dependencies.build[1].name, "make");
        assert_eq!(
            package.dependencies.build[1].version_req,
            Some(">=4.0".to_string())
        );
        assert_eq!(package.build.as_ref().unwrap().steps.len(), 3);
        assert_eq!(
            package.build.as_ref().unwrap().steps[0],
            "./configure --prefix=/usr/local"
        );
        assert_eq!(package.build.as_ref().unwrap().steps[1], "make -j$(nproc)");
        assert_eq!(package.build.as_ref().unwrap().steps[2], "make install");
    }
}
