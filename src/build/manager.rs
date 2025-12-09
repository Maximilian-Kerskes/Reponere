use std::{path::Path, vec};

pub enum PackageManagerError {
    UnknownManager,
    FailedInstall,
    FailedUninstall,
}

#[derive(PartialEq)]
pub enum ManagerKind {
    Pacman,
    Apt,
    Dnf,
}

pub struct PackageManager {
    pub kind: ManagerKind,
    sudo: bool,
    default_install_flags: Vec<&'static str>,
    default_uninstall_flags: Vec<&'static str>,
}

impl PackageManager {
    fn new(kind: ManagerKind, sudo: bool) -> Self {
        let default_install_flags = match kind {
            ManagerKind::Pacman => vec!["-S", "--noconfirm"],
            ManagerKind::Apt => vec!["install", "-y"],
            ManagerKind::Dnf => vec!["install", "-y"],
        };
        let default_uninstall_flags = match kind {
            ManagerKind::Pacman => vec!["-R", "--noconfirm"],
            ManagerKind::Apt => vec!["uninstall", "-y"],
            ManagerKind::Dnf => vec!["uninstall", "-y"],
        };

        PackageManager {
            kind,
            sudo,
            default_install_flags,
            default_uninstall_flags,
        }
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

    pub fn install(&self, package: &str) -> Result<(), PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.manager_string());
        cmd.extend(self.default_install_flags.iter().copied());
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

    pub fn uninstall(&self, package: &str) -> Result<(), PackageManagerError> {
        let mut cmd = self.command_prefix();
        cmd.push(self.manager_string());
        cmd.extend(self.default_uninstall_flags.iter().copied());
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
}

#[cfg(test)]
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
    pub fn test_install_and_uninstall() {
        let manager = PackageManager::get_package_manager(true).unwrap();

        let result = manager.install("minicom");
        assert!(
            result.is_ok(),
            "Expected install to succeed, got {result:?}",
        );

        let result = manager.install("asfd");
        assert!(
            result.is_err(),
            "Expected install to fail, got {result:?}",
        );

        let result = manager.uninstall("minicom");
        assert!(
            result.is_ok(),
            "Expected uninstall to succeed, got {result:?}"
        );

        let result = manager.uninstall("asfd");
        assert!(
            result.is_err(),
            "Expected uninstall to fail, got {:?}",
            result
        );
    }
}
