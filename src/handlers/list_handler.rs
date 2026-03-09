use crate::{handlers::events::ListEvent, util::context::Context};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListError {
    #[error("package not found: {0}")]
    PackageNotFound(String),
}


pub fn run<F: FnMut(ListEvent)>(
    ctx: &Context,
    packages: Vec<String>,
    available: bool,
    progress: &mut F,
) -> Result<(), ListError> {
    if available {
        progress(ListEvent::Available);
        if packages.is_empty() {
            for (name, package) in ctx.registry.get_packages() {
                progress(ListEvent::AvailablePackage(
                    name.clone(),
                    package.latest.clone(),
                ));
            }
            return Ok(());
        }
        for package_name in packages {
            match ctx.registry.get_package(&package_name) {
                Some(package) => progress(ListEvent::AvailablePackage(
                    package_name.clone(),
                    package.latest.clone(),
                )),
                None => return Err(ListError::PackageNotFound(package_name)),
            }
        }
        return Ok(());
    }

    progress(ListEvent::Installed);

    if packages.is_empty() {
        let packages = ctx.tracker.get_packages();
        for (name, package) in packages {
            progress(ListEvent::InstalledPackage(
                name.clone(),
                package.version.clone(),
            ));
        }
        return Ok(());
    }

    for package in packages {
        match ctx.tracker.get_package(&package) {
            Some(package) => progress(ListEvent::InstalledPackage(
                package.name.clone(),
                package.version.clone(),
            )),
            None => return Err(ListError::PackageNotFound(package)),
        }
    }

    Ok(())
}
