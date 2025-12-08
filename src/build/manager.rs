use std::path::Path;

#[derive(Debug)]
pub enum PackageManagerError {
    UnknownManager,
    FailedInstall,
}

#[derive(Debug, PartialEq)]
pub enum ManagerKind {
    Pacman,
    Apt,
    Dnf,
}

pub struct PackageManager {
    pub kind: ManagerKind,
    sudo: bool,
    default_flags: Vec<&'static str>,
}

impl PackageManager {
    fn new(kind: ManagerKind, sudo: bool) -> Self {
        let default_flags = match kind {
            ManagerKind::Pacman => vec!["-S", "--noconfirm"],
            ManagerKind::Apt => vec!["install", "-y"],
            ManagerKind::Dnf => vec!["install", "-y"],
        };

        PackageManager {
            kind,
            sudo,
            default_flags,
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

    pub fn install(&mut self, package: &str) -> Result<(), PackageManagerError> {
        let mut cmd = self.command_prefix();
        let (binary, flags) = match self.kind {
            ManagerKind::Pacman => ("pacman", &self.default_flags),
            ManagerKind::Apt => ("apt", &self.default_flags),
            ManagerKind::Dnf => ("dnf", &self.default_flags),
        };
        cmd.push(binary);
        cmd.extend(flags.iter().copied());
        cmd.push(package);

        println!("Running: {:?}", cmd);
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
        let manager = PackageManager::get_package_manager(true).unwrap();
        // depends on where its being build
        assert_eq!(manager.kind, ManagerKind::Apt);
    }

    #[test]
    pub fn test_install() {
        let mut manager = PackageManager::get_package_manager(true).unwrap();

        let result = manager.install("cmake");
        assert!(
            result.is_ok(),
            "Expected install to succeed, got {:?}",
            result
        );

        let result = manager.install("asfd");
        assert!(
            result.is_err(),
            "Expected install to fail, got {:?}",
            result
        );
    }
}
