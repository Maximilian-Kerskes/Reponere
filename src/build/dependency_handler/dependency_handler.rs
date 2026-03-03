use std::fmt;

use crate::build::dependency_handler::version::VersionRequirement;
use crate::build::package::package::{Dependencies, Dependency};
use crate::build::package_manager::manager::{PackageManagerApi, PackageManagerError};
use crate::handlers::install_handler::InstallEvent;

pub enum DependencyError {
    InstallFailed {
        dependency: String,
        source: PackageManagerError,
    },
    InstalledVersionCheckFailed {
        dependency: String,
        source: PackageManagerError,
    },
    AvailableVersionCheckFailed {
        dependency: String,
        source: PackageManagerError,
    },
}

impl fmt::Display for DependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyError::InstallFailed { dependency, source } => {
                write!(f, "Failed to install {dependency}: {source}")
            }
            DependencyError::InstalledVersionCheckFailed { dependency, source } => {
                write!(
                    f,
                    "Failed to check installed version of {dependency}: {source}"
                )
            }
            DependencyError::AvailableVersionCheckFailed { dependency, source } => {
                write!(
                    f,
                    "Failed to check available versions of {dependency}: {source}"
                )
            }
        }
    }
}

pub struct DependencyHandler<'a, PM: PackageManagerApi> {
    package_manager: &'a PM,
    dependencies: Dependencies,
}

impl<'a, PM: PackageManagerApi> DependencyHandler<'a, PM> {
    pub fn new(package_manager: &'a PM, dependencies: Dependencies) -> Self {
        DependencyHandler {
            package_manager,
            dependencies,
        }
    }

    pub fn install_runtime_dependencies<F: FnMut(InstallEvent)>(
        &self,
        errors: &mut Vec<DependencyError>,
        progress: &mut F,
    ) {
        for dependency in &self.dependencies.runtime {
            if self.dependency_needs_installing(dependency, errors) {
                progress(InstallEvent::InstallingDependency {
                    name: dependency.name.clone(),
                });
                self.install_dependency(dependency, errors);
            }
            progress(InstallEvent::DependencyAlreadyInstalled {
                name: dependency.name.clone(),
            });
        }
    }

    pub fn install_build_dependencies<F: FnMut(InstallEvent)>(
        &self,
        errors: &mut Vec<DependencyError>,
        progress: &mut F,
    ) -> Vec<String> {
        let mut installed = Vec::new();

        for dependency in &self.dependencies.build {
            if self.dependency_needs_installing(dependency, errors) {
                progress(InstallEvent::InstallingDependency {
                    name: dependency.name.clone(),
                });

                match self.package_manager.install(&dependency.name) {
                    Ok(()) => installed.push(dependency.name.clone()),
                    Err(e) => {
                        errors.push(DependencyError::InstallFailed {
                            dependency: dependency.name.clone(),
                            source: e,
                        });
                    }
                }
            }
            progress(InstallEvent::DependencyAlreadyInstalled {
                name: dependency.name.clone(),
            })
        }

        installed
    }

    fn dependency_needs_installing(
        &self,
        dependency: &Dependency,
        errors: &mut Vec<DependencyError>,
    ) -> bool {
        match self.package_manager.get_installed_version(&dependency.name) {
            Ok(Some(installed_version)) => {
                !self.check_version(&dependency.version_req, &installed_version)
            }
            Ok(None) => self.check_not_installed_availability(dependency, errors),
            Err(e) => {
                errors.push(DependencyError::InstalledVersionCheckFailed {
                    dependency: dependency.name.clone(),
                    source: e,
                });
                false
            }
        }
    }

    fn check_version(&self, version_requirement: &Option<String>, installed_version: &str) -> bool {
        match version_requirement {
            Some(req) => {
                let requirement = VersionRequirement::parse_requirement(req);
                requirement.matches(installed_version)
            }
            None => true,
        }
    }

    fn check_not_installed_availability(
        &self,
        dependency: &Dependency,
        errors: &mut Vec<DependencyError>,
    ) -> bool {
        match self.package_manager.get_available_version(&dependency.name) {
            Ok(Some(available_version)) => {
                self.check_version(&dependency.version_req, &available_version)
            }
            Ok(None) => {
                errors.push(DependencyError::AvailableVersionCheckFailed {
                    dependency: dependency.name.clone(),
                    source: PackageManagerError::NoVersionFound,
                });
                false
            }
            Err(e) => {
                errors.push(DependencyError::AvailableVersionCheckFailed {
                    dependency: dependency.name.clone(),
                    source: e,
                });
                false
            }
        }
    }

    fn install_dependency(&self, dependency: &Dependency, errors: &mut Vec<DependencyError>) {
        match self.package_manager.install(&dependency.name) {
            Ok(()) => {}
            Err(e) => {
                errors.push(DependencyError::InstallFailed {
                    dependency: dependency.name.clone(),
                    source: e,
                });
            }
        }
    }
}
