use std::{path::Path, vec};
use version_compare::Version;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageManagerError {
    #[error("Unknown package manager")]
    UnknownManager,
    #[error("Failed to install package: {0}")]
    FailedInstall(String),
    #[error("Failed to uninstall package: {0}")]
    FailedUninstall(String),
    #[error("Failed to get package version: {0}")]
    FailedGetVersion(String),
    #[error("No Package Version was found")]
    NoVersionFound,
    #[error("Failed to get reverse dependencies: {0}")]
    FailedGetReverseDependencies(String),
}

struct BackendConfig {
    cmd: &'static str,
    install_flags: &'static [&'static str],
    uninstall_flags: &'static [&'static str],
    get_installed_version_flags: &'static [&'static str],
    get_available_version_flags: &'static [&'static str],
    get_reverse_dependency_flags: &'static [&'static str],
}

const PACMAN_CONFIG: BackendConfig = BackendConfig {
    cmd: "pacman",
    install_flags: &["-S", "--noconfirm"],
    uninstall_flags: &["-R", "--noconfirm"],
    get_installed_version_flags: &["-Q"],
    get_available_version_flags: &["-Si"],
    get_reverse_dependency_flags: &["-Qi"],
};

const APT_CONFIG: BackendConfig = BackendConfig {
    cmd: "apt",
    install_flags: &["install", "-y"],
    uninstall_flags: &["remove", "-y"],
    // TODO
    // check if this is working
    get_installed_version_flags: &["list", "--installed"],
    get_available_version_flags: &["list"],
    get_reverse_dependency_flags: &["rdepends", "--installed"],
};

const DNF_CONFIG: BackendConfig = BackendConfig {
    cmd: "dnf",
    install_flags: &["install", "-y"],
    uninstall_flags: &["uninstall", "-y"],
    get_installed_version_flags: &["list", "installed"],
    get_available_version_flags: &["list", "available"],
    get_reverse_dependency_flags: &["repoquery", "--whatrequires", "--installed"],
};

#[derive(PartialEq)]
pub enum ManagerKind {
    Pacman,
    Apt,
    Dnf,
}

pub trait PackageManagerApi {
    fn install(&self, package: &str) -> Result<(), PackageManagerError>;
    fn uninstall(&self, package: &str) -> Result<(), PackageManagerError>;
    fn get_installed_version(&self, package: &str) -> Result<Option<String>, PackageManagerError>;
    fn get_available_version(&self, package: &str) -> Result<Option<String>, PackageManagerError>;
    fn reverse_dependencies(&self, package: &str) -> Result<Vec<String>, PackageManagerError>;
}

pub struct PackageManager {
    kind: ManagerKind,
    config: BackendConfig,
    sudo: bool,
}

impl PackageManager {
    fn new(kind: ManagerKind, sudo: bool) -> Self {
        let config = match kind {
            ManagerKind::Pacman => PACMAN_CONFIG,
            ManagerKind::Apt => APT_CONFIG,
            ManagerKind::Dnf => DNF_CONFIG,
        };

        PackageManager { kind, config, sudo }
    }

    pub fn get_package_manager(sudo: bool) -> Result<PackageManager, PackageManagerError> {
        if Path::new("/usr/lib/pacman").exists() || Path::new("/var/lib/pacman").exists() {
            Ok(PackageManager::new(ManagerKind::Pacman, sudo))
        } else if Path::new("/usr/lib/apt").exists() || Path::new("/var/lib/apt").exists() {
            Ok(PackageManager::new(ManagerKind::Apt, sudo))
        } else if Path::new("/usr/lib/dnf").exists() || Path::new("/var/lib/dnf").exists() {
            Ok(PackageManager::new(ManagerKind::Dnf, sudo))
        } else {
            Err(PackageManagerError::UnknownManager)
        }
    }

    fn command_prefix(&self) -> Vec<&str> {
        if self.sudo { vec!["sudo"] } else { vec![] }
    }

    fn parse_version(&self, input: &str) -> Option<String> {
        input
            .split_whitespace()
            .find(|word| Version::from(word).is_some())
            .map(|s| s.to_string())
    }

    fn parse_dependency(&self, input: &str) -> Result<Vec<String>, PackageManagerError> {
        let mut deps = Vec::new();

        match self.kind {
            ManagerKind::Pacman => {
                for line in input.lines() {
                    if !line.to_ascii_lowercase().starts_with("required by") {
                        continue;
                    }

                    let Some(dep_part) = line.splitn(2, ':').nth(1) else {
                        continue;
                    };

                    for dep in dep_part.split_whitespace() {
                        deps.push(dep.to_string());
                    }
                }
            }
            ManagerKind::Apt => {
                for line in input.lines() {
                    let line = line.trim();

                    if !line.to_ascii_lowercase().starts_with("depends") {
                        continue;
                    }

                    let Some(dep_part) = line.splitn(2, ':').nth(1) else {
                        continue;
                    };

                    let Some(dep) = dep_part.split_whitespace().next() else {
                        continue;
                    };

                    deps.push(dep.to_string());
                }
            }
            ManagerKind::Dnf => {
                for line in input.lines() {
                    if let Some(dep) = line.split_whitespace().next() {
                        deps.push(dep.to_string());
                    }
                }
            }
        }

        Ok(deps)
    }
}

impl PackageManagerApi for PackageManager {
    fn install(&self, package: &str) -> Result<(), PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.config.cmd);
        cmd.extend(self.config.install_flags.iter().copied());
        cmd.push(package);

        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| PackageManagerError::FailedInstall(e.to_string()))?;

        if !output.status.success() {
            return Err(PackageManagerError::FailedInstall(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    fn uninstall(&self, package: &str) -> Result<(), PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.config.cmd);
        cmd.extend(self.config.uninstall_flags.iter().copied());
        cmd.push(package);

        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| PackageManagerError::FailedUninstall(e.to_string()))?;

        if !output.status.success() {
            return Err(PackageManagerError::FailedUninstall(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    fn get_installed_version(&self, package: &str) -> Result<Option<String>, PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.config.cmd);
        cmd.extend(self.config.get_installed_version_flags.iter().copied());
        cmd.push(package);

        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| PackageManagerError::FailedGetVersion(e.to_string()))?;
        let stdout: String = String::from_utf8_lossy(&output.stdout).into();

        Ok(self.parse_version(&stdout))
    }

    fn get_available_version(&self, package: &str) -> Result<Option<String>, PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.config.cmd);
        cmd.extend(self.config.get_available_version_flags.iter().copied());
        cmd.push(package);

        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| PackageManagerError::FailedGetVersion(e.to_string()))?;
        let stdout: String = String::from_utf8_lossy(&output.stdout).into();

        Ok(self.parse_version(&stdout))
    }

    fn reverse_dependencies(&self, package: &str) -> Result<Vec<String>, PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.config.cmd);
        cmd.extend(self.config.get_reverse_dependency_flags.iter().copied());
        cmd.push(package);

        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| PackageManagerError::FailedGetReverseDependencies(e.to_string()))?;
        let stdout: String = String::from_utf8_lossy(&output.stdout).into();

        self.parse_dependency(&stdout)
    }
}

mod tests {
    use super::*;

    #[test]
    pub fn test_get_package_manager() {
        let manager_result = PackageManager::get_package_manager(true);
        match manager_result {
            Ok(m) => {
                assert!(matches!(
                    m.kind,
                    ManagerKind::Pacman | ManagerKind::Apt | ManagerKind::Dnf
                ));
            }
            Err(PackageManagerError::UnknownManager) => {
                panic!("No supported package manager found on this system.");
            }
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_parse_version_finds_first_semver() {
        let pm = PackageManager::new(ManagerKind::Pacman, false);

        let input = "cmake 3.28.1-1 (x86_64)";
        let version = pm.parse_version(input);

        assert_eq!(version, Some("3.28.1-1".to_string()));
    }

    #[test]
    fn test_parse_version_returns_none_if_missing() {
        let pm = PackageManager::new(ManagerKind::Pacman, false);

        let input = "no version here";
        let version = pm.parse_version(input);

        assert_eq!(version, None);
    }

    #[test]
    fn test_parse_dependency_pacman_required_by() {
        let pm = PackageManager::new(ManagerKind::Pacman, false);

        let input = "Name: libfoo\nVersion: 1.0.0\nDepends On: pkg1 pkg2 pkg3";

        let deps = pm.parse_dependency(input).unwrap();

        assert_eq!(deps.len(), 3);
        assert!(deps.contains(&"pkg1".to_string()));
        assert!(deps.contains(&"pkg2".to_string()));
        assert!(deps.contains(&"pkg3".to_string()));
    }

    #[test]
    fn test_parse_dependency_apt_depends() {
        let pm = PackageManager::new(ManagerKind::Apt, false);

        let input = "Package: foo\nDepends: libc6 (>= 2.34), libstdc++6, zlib1g";

        let deps = pm.parse_dependency(input).unwrap();

        assert_eq!(deps.len(), 3);
        assert!(deps.contains(&"libc6".to_string()));
        assert!(deps.contains(&"libstdc++6".to_string()));
        assert!(deps.contains(&"zlib1g".to_string()));
    }

    #[test]
    fn test_parse_dependency_empty() {
        let pm = PackageManager::new(ManagerKind::Pacman, false);

        let input = "Name: foo\n Version: 1.0";

        let deps = pm.parse_dependency(input).unwrap();

        assert!(deps.is_empty());
    }
}
