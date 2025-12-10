use crate::build::manager::PackageManager;
use crate::build::package::Dependencies;

pub struct DependencyHandler {
    package_manager: PackageManager,
    dependencies: Dependencies
}

impl DependencyHandler {
    pub fn new(package_manager: PackageManager, dependencies: Dependencies) -> Self {
        DependencyHandler {
            package_manager,
            dependencies
        }
    }

    pub fn install_dependencies(&self) {
        for dependency in &self.dependencies.build {
            let mut dep_name = dependency.name.clone();
            self.package_manager.install(&dependency.name);
        }
    }
}
