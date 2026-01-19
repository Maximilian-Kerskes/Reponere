mod tests {
    use reponere::build::dependency_handler::version::VersionRequirement;

    #[test]
    fn test_matches() {
        let req = VersionRequirement::parse_requirement("==1.2.3");
        assert!(req.matches("1.2.3"));
        assert!(!req.matches("1.2.4"));

        let req = VersionRequirement::parse_requirement(">1.2.3");
        assert!(req.matches("1.2.4"));
        assert!(!req.matches("1.2.3"));

        let req = VersionRequirement::parse_requirement(">=1.2.3");
        assert!(req.matches("1.2.3"));
        assert!(req.matches("1.2.4"));
        assert!(!req.matches("1.2.2"));

        let req = VersionRequirement::parse_requirement("<2.0.0");
        assert!(req.matches("1.9.9"));
        assert!(!req.matches("2.0.0"));

        let req = VersionRequirement::parse_requirement("<=2.0.0");
        assert!(req.matches("2.0.0"));
        assert!(req.matches("1.9.9"));
        assert!(!req.matches("2.0.1"));

        // No operator defaults to ==
        let req = VersionRequirement::parse_requirement("3.3.3");
        assert!(req.matches("3.3.3"));
        assert!(!req.matches("3.3.4"));
    }

    #[test]
    fn test_invalid_version_strings() {
        let req = VersionRequirement::parse_requirement(">=1.0.0");
        assert!(!req.matches("not-a-version"));
    }
}
