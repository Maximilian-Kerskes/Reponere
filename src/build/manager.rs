use std::{fmt, path::Path, vec};
use version_compare::Version;

#[derive(Debug)]
pub enum PackageManagerError {
    UnknownManager,
    FailedInstall,
    FailedUninstall,
    FailedGetVersion,
    NoVersionFound,
}

impl fmt::Display for PackageManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManagerError::UnknownManager => write!(f, "Unknown package manager"),
            PackageManagerError::FailedInstall => write!(f, "Failed to install package"),
            PackageManagerError::FailedUninstall => write!(f, "Failed to uninstall package"),
            PackageManagerError::FailedGetVersion => write!(f, "Failed to get package version"),
            PackageManagerError::NoVersionFound => write!(f, "No Package Version was found"),
        }
    }
}

struct BackendConfig {
    cmd: &'static str,
    install_flags: &'static [&'static str],
    uninstall_flags: &'static [&'static str],
    get_installed_version_flags: &'static [&'static str],
    get_available_version_flags: &'static [&'static str],
}

const PACMAN_CONFIG: BackendConfig = BackendConfig {
    cmd: "pacman",
    install_flags: &["-S", "--noconfirm"],
    uninstall_flags: &["-R", "--noconfirm"],
    get_installed_version_flags: &["-Q"],
    get_available_version_flags: &["-Si"],
};

const APT_CONFIG: BackendConfig = BackendConfig {
    cmd: "apt",
    install_flags: &["install", "-y"],
    uninstall_flags: &["remove", "-y"],
    // TODO
    // check if this is working
    get_installed_version_flags: &["list", "--installed"],
    get_available_version_flags: &["list"],
};

const DNF_CONFIG: BackendConfig = BackendConfig {
    cmd: "dnf",
    install_flags: &["install", "-y"],
    uninstall_flags: &["uninstall", "-y"],
    get_installed_version_flags: &["list", "installed"],
    get_available_version_flags: &["list", "available"],
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

    fn manager_string(&self) -> &'static str {
        match self.kind {
            ManagerKind::Pacman => "pacman",
            ManagerKind::Apt => "apt",
            ManagerKind::Dnf => "dnf",
        }
    }

    fn parse_version(&self, input: &str) -> Option<String> {
        input
            .split_whitespace()
            .find(|word| Version::from(word).is_some())
            .map(|s| s.to_string())
    }
}

impl PackageManagerApi for PackageManager {
    fn install(&self, package: &str) -> Result<(), PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.manager_string());
        cmd.extend(self.config.install_flags.iter().copied());
        cmd.push(package);

        println!("Running: {cmd:?}");
        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|_| PackageManagerError::FailedInstall)?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            return Err(PackageManagerError::FailedInstall);
        }

        Ok(())
    }

    fn uninstall(&self, package: &str) -> Result<(), PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.manager_string());
        cmd.extend(self.config.uninstall_flags.iter().copied());
        cmd.push(package);

        println!("Running: {cmd:?}");
        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|_| PackageManagerError::FailedInstall)?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            return Err(PackageManagerError::FailedInstall);
        }

        Ok(())
    }

    fn get_installed_version(&self, package: &str) -> Result<Option<String>, PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.manager_string());
        cmd.extend(self.config.get_installed_version_flags.iter().copied());
        cmd.push(package);

        println!("Running: {cmd:?}");
        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|_| PackageManagerError::FailedGetVersion)?;
        let stdout: String = String::from_utf8_lossy(&output.stdout).into();
        println!("{}", stdout);
        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(self.parse_version(&stdout))
    }

    fn get_available_version(&self, package: &str) -> Result<Option<String>, PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.manager_string());
        cmd.extend(self.config.get_available_version_flags.iter().copied());
        cmd.push(package);

        println!("Running: {cmd:?}");
        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|_| PackageManagerError::FailedInstall)?;
        let stdout: String = String::from_utf8_lossy(&output.stdout).into();
        println!("{}", stdout);
        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(self.parse_version(&stdout))
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
}
