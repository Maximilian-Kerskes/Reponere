use crate::build::manager::PackageManager;
use crate::build::package::Dependencies;
use version_compare::Version;

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

    pub fn install_dependencies(&self) {
        for dependency in &self.dependencies.build {
            let should_install = if let Some(version_req) = dependency.version_req.as_ref() {
                let version_requirement = VersionRequirement::parse_requirement(version_req);

                match self.package_manager.get_installed_version(&dependency.name) {
                    Ok(installed_version) => {
                        // Only install if installed version does NOT satisfy the requirement
                        !version_requirement.matches(&installed_version)
                    }
                    Err(_) => true,
                }
            } else {
                true
            };

            if !should_install {
                continue;
            }
            if let Err(e) = self.package_manager.install(&dependency.name) {
                eprintln!("Failed to install dependency {}: {}", &dependency.name, e);
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
