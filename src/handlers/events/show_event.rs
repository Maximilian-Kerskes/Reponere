use crate::handlers::events::event::Event;

pub enum ShowEvent {
    LookingUpPackage { name: String },
    PackageNotFound { name: String },

    ShowingPackage { name: String, latest: String },
    AvailableVersions { versions: Vec<String> },

    InstalledVersion { version: String },
    InstalledPath { path: String },

    StatusUpToDate,
    StatusOutdated { latest: String },
    StatusNotInstalled,

    BuildFile { path: String },

    Finished,
}

impl Event for ShowEvent {
    fn message(&self) -> String {
        match self {
            ShowEvent::LookingUpPackage { name } => {
                format!("==> Looking up package {name}")
            }
            ShowEvent::PackageNotFound { name } => {
                format!("==> Package {name} not found")
            }
            ShowEvent::ShowingPackage { name, latest } => {
                format!("-> Package {name}@{latest}")
            }
            ShowEvent::AvailableVersions { versions } => {
                format!("-> Available versions: {versions:?}")
            }
            ShowEvent::InstalledVersion { version } => {
                format!("-> Installed version: {version}")
            }
            ShowEvent::InstalledPath { path } => format!("-> Installed path: {path}"),
            ShowEvent::StatusUpToDate => "==> Status: Up-to-date".to_string(),
            ShowEvent::StatusOutdated { latest } => {
                format!("==> Status: Outdated (latest is {latest})")
            }
            ShowEvent::StatusNotInstalled => "==> Status: Not installed".to_string(),
            ShowEvent::BuildFile { path } => format!("-> Build file: {path}"),
            ShowEvent::Finished => "==> Finished".to_string(),
        }
    }
}
