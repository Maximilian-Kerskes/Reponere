use std::{collections::HashMap, fs, path::Path};

use crate::build::dependency_handler::version::is_newer;

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    packages: HashMap<String, PackageEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageEntry {
    pub releases: HashMap<String, Release>,
    pub latest: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    build_file: String,
}

impl Release {
    pub fn build_file(&self) -> &str {
        &self.build_file
    }
}

impl Registry {
    fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    fn load_from_file(path: &Path) -> Result<Self, std::io::Error> {
        let data = fs::read_to_string(path)?;
        let registry: Registry = serde_json::from_str(&data)?;
        Ok(registry)
    }

    fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn sync_from_directory(dir: &Path) -> Result<Self, std::io::Error> {
        let mut registry = Registry::new();

        for entry in WalkDir::new(dir)
            .min_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.file_name().and_then(|s| s.to_str()) != Some("package_build.yaml") {
                continue;
            }

            let version_dir = path.parent().unwrap();
            let version = version_dir.file_name().unwrap().to_str().unwrap();

            let package_dir = version_dir.parent().unwrap();
            let package_name = package_dir.file_name().unwrap().to_str().unwrap();

            let release = Release {
                build_file: path.to_str().unwrap().to_string(),
            };

            let entry = registry
                .packages
                .entry(package_name.to_string())
                .or_insert_with(|| PackageEntry {
                    releases: HashMap::new(),
                    latest: version.to_string(),
                });
            entry.releases.insert(version.to_string(), release);

            if is_newer(version, &entry.latest) {
                entry.latest = version.to_string();
            }
        }
        Ok(registry)
    }

    pub fn load_or_sync(index: &Path, registry_dir: &Path) -> Self {
        match Registry::load_from_file(index) {
            Ok(registry) => registry,
            Err(_) => {
                let registry = Registry::sync_from_directory(registry_dir).unwrap();
                registry.save_to_file(index).unwrap();
                registry
            }
        }
    }

    pub fn resync_from_directory_and_save(index: &Path, registry_dir: &Path) -> Self {
        let registry = Registry::sync_from_directory(registry_dir).unwrap();
        registry.save_to_file(index).unwrap();
        registry
    }

    pub fn resolve_release(&self, name: &str, version: Option<&str>) -> Option<&Release> {
        let package = self.packages.get(name)?;

        let version = match version {
            Some(version) => version,
            None => &package.latest,
        };

        package.releases.get(version)
    }
    pub fn get_package(&self, name: &str) -> Option<&PackageEntry> {
        self.packages.get(name)
    }

    pub fn get_packages(&self) -> &HashMap<String, PackageEntry> {
        &self.packages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_sync_creates_registry() {
        let dir = tempdir().unwrap();

        // create sample package structure
        fs::create_dir_all(dir.path().join("mypkg/1.0.0")).unwrap();
        fs::write(
            dir.path().join("mypkg/1.0.0/package_build.yaml"),
            "build: []",
        )
        .unwrap();

        let registry = Registry::sync_from_directory(dir.path()).unwrap();

        assert!(registry.packages.contains_key("mypkg"));

        let entry = registry.packages.get("mypkg").unwrap();
        assert_eq!(entry.latest, "1.0.0");
        assert!(entry.releases.contains_key("1.0.0"));
    }

    #[test]
    fn test_load_and_save_roundtrip() {
        let dir = tempdir().unwrap();
        let index = dir.path().join("index.json");

        let mut registry = Registry::new();
        registry.packages.insert(
            "pkg".to_string(),
            PackageEntry {
                latest: "1.0.0".to_string(),
                releases: HashMap::new(),
            },
        );

        registry.save_to_file(&index).unwrap();

        let loaded = Registry::load_from_file(&index).unwrap();
        assert!(loaded.packages.contains_key("pkg"));
    }

    #[test]
    fn test_load_or_sync_fallback() {
        let dir = tempdir().unwrap();
        let index = dir.path().join("index.json");

        // no index exists → should sync
        fs::create_dir_all(dir.path().join("pkg/2.0.0")).unwrap();
        fs::write(dir.path().join("pkg/2.0.0/package_build.yaml"), "build: []").unwrap();

        let registry = Registry::load_or_sync(&index, dir.path());

        assert!(registry.packages.contains_key("pkg"));
        assert_eq!(registry.packages.get("pkg").unwrap().latest, "2.0.0");
    }

    #[test]
    fn test_latest_semver_logic() {
        let dir = tempdir().unwrap();

        fs::create_dir_all(dir.path().join("pkg/1.0.0")).unwrap();
        fs::create_dir_all(dir.path().join("pkg/2.0.0")).unwrap();

        fs::write(dir.path().join("pkg/1.0.0/package_build.yaml"), "").unwrap();
        fs::write(dir.path().join("pkg/2.0.0/package_build.yaml"), "").unwrap();

        let registry = Registry::sync_from_directory(dir.path()).unwrap();

        let entry = registry.packages.get("pkg").unwrap();
        assert_eq!(entry.latest, "2.0.0");
    }
}
