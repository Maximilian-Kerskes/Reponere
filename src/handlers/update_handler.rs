use crate::{
    build::{
        dependency_handler::version::is_newer,
    },
    handlers::events::UpdateEvent,
    util::context::Context,
};

#[derive(Debug, PartialEq, Eq)]
pub struct UpdatePlan {
    pub name: String,
    pub installed_version: String,
    pub latest_version: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    UpdateAvailable(UpdatePlan),
    AlreadyUpToDate,
    AheadOfRegistry,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateError {
    PackageNotInstalled(String),
    PackageNotFound(String),
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateError::PackageNotInstalled(name) => write!(f, "package not installed: {name}"),
            UpdateError::PackageNotFound(name) => write!(f, "package not found: {name}"),
        }
    }
}

pub fn run<F: FnMut(UpdateEvent)>(
    ctx: &Context,
    package: &str,
    progress: &mut F,
) -> Result<UpdateStatus, UpdateError> {
    progress(UpdateEvent::CheckingPackage {
        name: package.to_string(),
    });

    let installed = ctx
        .tracker
        .get_package(package)
        .ok_or_else(|| UpdateError::PackageNotInstalled(package.to_string()))?;

    let entry = ctx
        .registry
        .get_package(package)
        .ok_or_else(|| UpdateError::PackageNotFound(package.to_string()))?;

    if is_newer(&entry.latest, &installed.version) {
        progress(UpdateEvent::UpdateAvailable {
            name: package.to_string(),
            installed: installed.version.clone(),
            latest: entry.latest.clone(),
        });

        return Ok(UpdateStatus::UpdateAvailable(UpdatePlan {
            name: package.to_string(),
            installed_version: installed.version.clone(),
            latest_version: entry.latest.clone(),
        }));
    }

    if is_newer(&installed.version, &entry.latest) {
        progress(UpdateEvent::PackageAheadOfRegistry {
            name: package.to_string(),
            installed: installed.version.clone(),
            latest: entry.latest.clone(),
        });
        return Ok(UpdateStatus::AheadOfRegistry);
    }

    progress(UpdateEvent::PackageAlreadyUpToDate {
        name: package.to_string(),
        version: installed.version.clone(),
    });

    Ok(UpdateStatus::AlreadyUpToDate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use crate::build::registry::registry_handler::Registry;
    use crate::handlers::events::event::Event;

    fn context_with(package_name: &str, installed_version: &str, latest_version: &str) -> Context {
        let temp = tempfile::tempdir().unwrap();
        let registry_root = temp.path().join("registry");
        let package_dir = registry_root.join(package_name).join(latest_version);
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(package_dir.join("package_build.yaml"), "build: []").unwrap();

        let mut tracker = crate::build::package_tracker::package_tracker::PackageTracker::default();
        tracker.add_package(crate::build::package::package::InstalledPackage {
            name: package_name.to_string(),
            version: installed_version.to_string(),
            install_path: "/tmp/test".to_string(),
            dependencies: Vec::new(),
        });

        Context {
            config: crate::util::config::Config {
                index_path: temp.path().join("index.json"),
                registry_path: registry_root.clone(),
                packages_path: temp.path().join("packages.json"),
            },
            registry: Registry::load_or_sync(&temp.path().join("index.json"), &registry_root),
            tracker,
        }
    }

    #[test]
    fn returns_update_plan_for_outdated_package() {
        let ctx = context_with("rg", "14.0.0", "15.1.0");
        let mut events = Vec::new();

        let result = run(&ctx, "rg", &mut |event| events.push(event.message())).unwrap();

        assert_eq!(
            result,
            UpdateStatus::UpdateAvailable(UpdatePlan {
                name: "rg".to_string(),
                installed_version: "14.0.0".to_string(),
                latest_version: "15.1.0".to_string(),
            })
        );
        assert!(events.iter().any(|msg| msg.contains("Update available")));
    }

    #[test]
    fn returns_up_to_date_for_matching_versions() {
        let ctx = context_with("rg", "15.1.0", "15.1.0");

        let result = run(&ctx, "rg", &mut |_| {}).unwrap();

        assert_eq!(result, UpdateStatus::AlreadyUpToDate);
    }

    #[test]
    fn errors_when_package_is_not_installed() {
        let ctx = context_with("rg", "15.1.0", "15.1.0");

        let result = run(&ctx, "fd", &mut |_| {});

        assert_eq!(
            result,
            Err(UpdateError::PackageNotInstalled("fd".to_string()))
        );
    }
}
