use std::{path::Path, process::Command};
use thiserror::Error;

use crate::{build::package::package::Build, handlers::install_handler::InstallEvent};

#[derive(Debug, Error)]
pub enum BuildHandlerError {
    #[error("Build step failed: {0}")]
    BuildStepFailed(String),

    #[error("Failed to spawn process: {0}")]
    SpawnError(#[from] std::io::Error),
}

pub struct BuildHandler {
    pub build_steps: Build,
}

impl BuildHandler {
    pub fn new(build: Build) -> Self {
        BuildHandler { build_steps: build }
    }

    pub fn run_build_steps<F: FnMut(InstallEvent)>(
        &self,
        path: &Path,
        progress: &mut F,
    ) -> Result<(), BuildHandlerError> {
        for step in &self.build_steps.steps {
            progress(InstallEvent::BuildStep { step: step.clone() });
            let status = Command::new("sh")
                .arg("-c")
                .arg(step)
                .current_dir(path)
                .status()
                .map_err(|e| BuildHandlerError::SpawnError(e))?;

            if !status.success() {
                return Err(BuildHandlerError::BuildStepFailed(step.to_string()));
            }
        }
        Ok(())
    }
}
