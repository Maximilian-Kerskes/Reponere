use crate::handlers::events::event::Event;

pub enum UpdateEvent {
    CheckingPackage { name: String },
    PackageNotInstalled { name: String },
    PackageNotFound { name: String },
    PackageAlreadyUpToDate { name: String, version: String },
    PackageAheadOfRegistry {
        name: String,
        installed: String,
        latest: String,
    },
    UpdateAvailable {
        name: String,
        installed: String,
        latest: String,
    },
}

impl Event for UpdateEvent {
    fn message(&self) -> String {
        match self {
            UpdateEvent::CheckingPackage { name } => format!("==> Checking {name}"),
            UpdateEvent::PackageNotInstalled { name } => {
                format!("==> Package {name} is not installed")
            }
            UpdateEvent::PackageNotFound { name } => {
                format!("==> Package {name} was not found in the registry")
            }
            UpdateEvent::PackageAlreadyUpToDate { name, version } => {
                format!("==> Package {name} is already up to date ({version})")
            }
            UpdateEvent::PackageAheadOfRegistry {
                name,
                installed,
                latest,
            } => {
                format!(
                    "==> Package {name} is newer locally ({installed}) than the registry latest ({latest})"
                )
            }
            UpdateEvent::UpdateAvailable {
                name,
                installed,
                latest,
            } => {
                format!("==> Update available for {name}: {installed} -> {latest}")
            }
        }
    }
}
