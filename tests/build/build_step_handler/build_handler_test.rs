mod tests {
    use reponere::build::build_step_handler::build_handler::{BuildHandler, BuildHandlerError};
    use reponere::build::package::package::Build;

    #[test]
    fn test_run_build_steps_success() {
        let build = Build {
            steps: vec!["echo hello".to_string()],
        };
        let handler = BuildHandler::new(build);

        let result = handler.run_build_steps();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_build_steps_failure() {
        let build = Build {
            steps: vec!["false".to_string()],
        };
        let handler = BuildHandler::new(build);

        let result = handler.run_build_steps();
        assert!(matches!(result, Err(BuildHandlerError::BuildStepFailed(_))));
    }

    #[test]
    fn test_run_multiple_steps() {
        let build = Build {
            steps: vec!["echo step1".to_string(), "echo step2".to_string()],
        };
        let handler = BuildHandler::new(build);

        let result = handler.run_build_steps();
        assert!(result.is_ok());
    }
}
