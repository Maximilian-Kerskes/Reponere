use crate::handlers::events::event::Event;

pub enum InstallEvent {
    InstallingDependencies,
    InstallingRunTimeDependencies { dependencies: Vec<String> },
    InstallingBuildDependencies { dependencies: Vec<String> },
    InstallingDependency { name: String },
    DependencyAlreadyInstalled { name: String },
    FetchingSource,
    BuildingSource,
    BuildStep { step: String },
    Cleanup,
    Finished,
}

impl Event for InstallEvent {
    fn message(&self) -> String {
        match self {
            InstallEvent::InstallingDependencies => "==> Installing dependencies".to_string(),
            InstallEvent::InstallingRunTimeDependencies { dependencies } => {
                format!("==> Installing runtime dependencies: {dependencies:?}")
            }
            InstallEvent::InstallingBuildDependencies { dependencies } => {
                format!("==> Installing build dependencies: {dependencies:?}")
            }
            InstallEvent::InstallingDependency { name } => {
                format!("-> installing dependency {name}...")
            }
            InstallEvent::DependencyAlreadyInstalled { name } => {
                format!("-> dependency {name} already installed")
            }
            InstallEvent::FetchingSource => "==> Fetching source".to_string(),
            InstallEvent::BuildingSource => "==> Building source".to_string(),
            InstallEvent::BuildStep { step } => format!("-> {step}"),
            InstallEvent::Cleanup => "==> Cleanup".to_string(),
            InstallEvent::Finished => "==> Finished".to_string(),
        }
    }
}
