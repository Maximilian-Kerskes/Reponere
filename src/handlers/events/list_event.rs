use crate::handlers::events::event::Event;

pub enum ListEvent {
    Available,
    AvailablePackage(String, String),
    Installed,
    InstalledPackage(String, String),
}

impl Event for ListEvent {
    fn message(&self) -> String {
        match self {
            ListEvent::Available => "==> Available packages:".to_string(),
            ListEvent::AvailablePackage(name, version) => {
                format!("-> {name}@{version}")
            }
            ListEvent::Installed => "==> Installed packages:".to_string(),
            ListEvent::InstalledPackage(name, version) => {
                format!("-> {name}@{version}")
            }
        }
    }
}
