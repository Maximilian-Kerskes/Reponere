use crate::build::manager::{PackageManager, PackageManagerError};
use crate::build::package::{Dependencies, Dependency};
use version_compare::Version;

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

pub struct DependencyHandler {
    package_manager: PackageManager,
    dependencies: Dependencies,
}

impl DependencyHandler {
    pub fn new(package_manager: PackageManager, dependencies: Dependencies) -> Self {
        DependencyHandler {
            package_manager,
            dependencies,
        }
    }

    pub fn install_dependencies(&self) -> Result<(), Vec<DependencyError>> {
        let mut errors = Vec::new();

        for dependency in &self.dependencies.build {
            if self.dependency_needs_installing(dependency, &mut errors) {
                self.install_dependency(dependency, &mut errors);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn dependency_needs_installing(
        &self,
        dependency: &Dependency,
        errors: &mut Vec<DependencyError>,
    ) -> bool {
        match self.package_manager.get_installed_version(&dependency.name) {
            Ok(Some(installed_version)) => {
                self.check_version(&dependency.version_req, &installed_version)
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

enum Operator {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
}

struct VersionRequirement {
    op: Operator,
    version: String,
}

impl VersionRequirement {
    fn new(op: Operator, version: String) -> Self {
        VersionRequirement { op, version }
    }

    pub fn parse_requirement(req: &str) -> Self {
        if let Some(rest) = req.strip_prefix(">=") {
            return Self::new(Operator::Ge, rest.to_string());
        }
        if let Some(rest) = req.strip_prefix("<=") {
            return Self::new(Operator::Le, rest.to_string());
        }
        if let Some(rest) = req.strip_prefix(">") {
            return Self::new(Operator::Gt, rest.to_string());
        }
        if let Some(rest) = req.strip_prefix("<") {
            return Self::new(Operator::Lt, rest.to_string());
        }
        if let Some(rest) = req.strip_prefix("==") {
            return Self::new(Operator::Eq, rest.to_string());
        }

        // Default to equality if no operator specified
        Self::new(Operator::Eq, req.to_string())
    }

    pub fn matches(&self, dep_version: &str) -> bool {
        let dep_version = match Version::from(dep_version) {
            Some(v) => v,
            None => return false,
        };

        let req_version = match Version::from(&self.version) {
            Some(v) => v,
            None => return false,
        };

        match self.op {
            Operator::Gt => dep_version > req_version,
            Operator::Ge => dep_version >= req_version,
            Operator::Lt => dep_version < req_version,
            Operator::Le => dep_version <= req_version,
            Operator::Eq => dep_version == req_version,
        }
    }
}
