use crate::handlers::events::event::Event;

pub enum UninstallEvent {
    UninstallingDependencies,
    UninstallingDependency { name: String },
    DependencyAlreadyUninstalled { name: String },
    RemovingPackageFiles,
    Cleanup,
    Finished,
}

impl Event for UninstallEvent {
    fn message(&self) -> String {
        match self {
            UninstallEvent::UninstallingDependencies => "==> Uninstalling dependencies".to_string(),
            UninstallEvent::UninstallingDependency { name } => {
                format!("-> uninstalling dependency {name}...")
            }
            UninstallEvent::DependencyAlreadyUninstalled { name } => {
                format!("-> dependency {name} already uninstalled")
            }
            UninstallEvent::RemovingPackageFiles => "==> Removing package files".to_string(),
            UninstallEvent::Cleanup => "==> Cleanup".to_string(),
            UninstallEvent::Finished => "==> Finished".to_string(),
        }
    }
}
