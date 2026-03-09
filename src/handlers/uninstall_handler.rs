use std::{fs, path::Path};

use crate::{
    build::{
        package::package::{Dependency, InstalledPackage},
        package_manager::manager::{PackageManager, PackageManagerApi},
        package_tracker::package_tracker::PackageTracker,
    },
    handlers::events::UninstallEvent,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UninstallError {
    #[error("already uninstalled")]
    AlreadyUninstalled,
    #[error("package manager error: {0}")]
    PackageManagerError(String),
    #[error("io error: {0}")]
    Io(std::io::Error),
}

#[derive(Debug, Default)]
pub struct UninstallPlan {
    pub package: InstalledPackage,
    pub remove_dependencies: Vec<Dependency>,
    pub keep_dependencies: Vec<Dependency>,
}

pub fn plan(tracker: &PackageTracker, package: &str) -> Result<UninstallPlan, UninstallError> {
    let installed = tracker
        .get_package(package)
        .ok_or(UninstallError::AlreadyUninstalled)?;

    let mut remove = Vec::new();
    let mut keep = Vec::new();

    for dep in &installed.dependencies {
        if is_dependency_used_by_others(dep) < 1 || tracker.dependency_usage_count(&dep.name) < 1 {
            remove.push(dep.clone());
        } else {
            keep.push(dep.clone());
        }
    }

    Ok(UninstallPlan {
        package: installed.clone(),
        remove_dependencies: remove,
        keep_dependencies: keep,
    })
}

pub fn execute<F: FnMut(UninstallEvent)>(
    tracker: &mut PackageTracker,
    plan: UninstallPlan,
    progress: &mut F,
) -> Result<(), UninstallError> {
    progress(UninstallEvent::UninstallingDependencies);
    uninstall_dependencies(plan.remove_dependencies, progress)?;

    progress(UninstallEvent::RemovingPackageFiles);
    remove_package_files(&plan.package)?;

    progress(UninstallEvent::Cleanup);
    tracker.remove_package(&plan.package.name);

    progress(UninstallEvent::Finished);

    Ok(())
}

fn uninstall_dependencies<F: FnMut(UninstallEvent)>(
    dependencies: Vec<Dependency>,
    progress: &mut F,
) -> Result<(), UninstallError> {
    let pm = PackageManager::get_package_manager(true)
        .map_err(|e| UninstallError::PackageManagerError(e.to_string()))?;

    for dependency in dependencies {
        progress(UninstallEvent::UninstallingDependency {
            name: dependency.name.clone(),
        });
        pm.uninstall(&dependency.name)
            .map_err(|e| UninstallError::PackageManagerError(e.to_string()))?;
    }
    Ok(())
}

fn is_dependency_used_by_others(dep: &Dependency) -> usize {
    let pm = PackageManager::get_package_manager(true).unwrap();
    pm.reverse_dependencies(&dep.name).unwrap().len()
}

fn remove_package_files(package: &InstalledPackage) -> Result<(), UninstallError> {
    let path = Path::new(&package.install_path);

    if !path.exists() {
        return Err(UninstallError::Io(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )));
    }

    if path.is_dir() {
        fs::remove_dir_all(path).map_err(UninstallError::Io)?;
    } else {
        fs::remove_file(path).map_err(UninstallError::Io)?;
    }

    Ok(())
}
