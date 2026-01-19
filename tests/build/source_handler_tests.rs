mod tests {
    use reponere::build::package::Source;
    use reponere::build::source_handler::{GitSource, GitSourceHandler, GitSourceHandlerError};

    #[test]
    fn test_from_source_valid() {
        let source = Source::Git {
            repo: "https://github.com/user/repo.git".into(),
            tag: Some("v1.0.0".into()),
            branch: None,
            commit: None,
        };

        let git_source = GitSource::from_source(&source).unwrap();
        assert_eq!(git_source.repo, "https://github.com/user/repo.git");
        assert_eq!(git_source.tag, Some("v1.0.0"));
        assert_eq!(git_source.branch, None);
        assert_eq!(git_source.commit, None);
    }

    #[test]
    fn test_from_source_multiple_fields_error() {
        let source = Source::Git {
            repo: "https://github.com/user/repo.git".into(),
            tag: Some("v1.0.0".into()),
            branch: Some("main".into()),
            commit: None,
        };

        let err = GitSource::from_source(&source).unwrap_err();
        match err {
            GitSourceHandlerError::InvalidSpecifications(msg) => {
                assert!(msg.contains("Only one of commit, tag, or branch"))
            }
            _ => panic!("Expected InvalidSpecifications error"),
        }
    }

    /// NOTE: This clones a public repo, so it requires internet access
    #[test]
    fn test_fetch_repo() {
        let source = Source::Git {
            repo: "https://github.com/Maximilian-Kerskes/Reponere".into(),
            tag: None,
            branch: Some("master".into()),
            commit: None,
        };

        let git_source = GitSource::from_source(&source).unwrap();
        let handler = GitSourceHandler::new(git_source);

        let tmp_dir = handler.fetch().unwrap();
        let path = tmp_dir.path();

        // Check that the directory exists and contains .git
        assert!(path.exists());
        assert!(path.join(".git").exists());
    }

    /// Test that HEAD is checked out if nothing is specified
    #[test]
    fn test_fetch_default_head() {
        let source = Source::Git {
            repo: "https://github.com/Maximilian-Kerskes/Reponere".into(),
            tag: None,
            branch: None,
            commit: None,
        };

        let git_source = GitSource::from_source(&source).unwrap();
        let handler = GitSourceHandler::new(git_source);

        let tmp_dir = handler.fetch().unwrap();
        let path = tmp_dir.path();

        assert!(path.exists());
        assert!(path.join(".git").exists());
    }
}
