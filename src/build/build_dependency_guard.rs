use crate::build::manager::PackageManagerApi;

pub struct BuildDependencyGuard<'a, PM: PackageManagerApi> {
    pub package_manager: &'a PM,
    pub installed: Vec<String>,
}

impl<'a, PM: PackageManagerApi> Drop for BuildDependencyGuard<'a, PM> {
    fn drop(&mut self) {
        for dep in &self.installed {
            let _ = self.package_manager.uninstall(dep);
        }
    }
}
