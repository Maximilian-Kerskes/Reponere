use crate::build::manager::{PackageManagerApi, PackageManagerError};
use crate::build::package::{Dependencies, Dependency};
use crate::build::version::VersionRequirement;

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

    pub fn install_runtime_dependencies(&self, errors: &mut Vec<DependencyError>) {
        for dependency in &self.dependencies.runtime {
            if self.dependency_needs_installing(dependency, errors) {
                self.install_dependency(dependency, errors);
            }
        }
    }

    pub fn install_build_dependencies(&self, errors: &mut Vec<DependencyError>) -> Vec<String> {
        let mut installed = Vec::new();

        for dependency in &self.dependencies.build {
            if self.dependency_needs_installing(dependency, errors) {
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
