use crate::build::package::*;
use std::{fs::File, io};

pub struct PackageParser {
    filename: String,
}

impl PackageParser {
    pub fn new<S: Into<String>>(filename: S) -> PackageParser {
        PackageParser {
            filename: filename.into(),
        }
    }

    pub fn parse(&self) -> Result<Package, Box<dyn std::error::Error>> {
        let file = File::open(&self.filename)?;
        let reader = io::BufReader::new(file);
        let package = serde_yml::from_reader(reader)?;
        Ok(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse() {
        let parser = PackageParser::new("./tests/package.yaml");
        let package = parser.parse().unwrap();
        assert_eq!(package.name, "mypackage");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.description, Some("package description".to_string()));
        assert_eq!(
            package.source,
            Source::Git {
                repo: "https://github.com/user/mypackage.git".to_string(),
                tag: Some("v1.0.0".to_string()),
                branch: Some("main".to_string()),
                commit: None
            }
        );
        assert_eq!(package.dependencies.runtime.len(), 2);
        assert_eq!(package.dependencies.runtime[0].name, "serde");
        assert_eq!(
            package.dependencies.runtime[0].version_req,
            Some(">=1.0".to_string())
        );
        assert_eq!(package.dependencies.runtime[1].name, "tokio");
        assert_eq!(
            package.dependencies.runtime[1].version_req,
            Some("1.25".to_string())
        );
        assert_eq!(package.dependencies.build.len(), 2);
        assert_eq!(package.dependencies.build[0].name, "cmake");
        assert_eq!(package.dependencies.build[1].name, "make");
        assert_eq!(
            package.dependencies.build[1].version_req,
            Some(">=4.0".to_string())
        );
        assert_eq!(package.build.as_ref().unwrap().steps.len(), 3);
        assert_eq!(
            package.build.as_ref().unwrap().steps[0],
            "./configure --prefix=/usr/local"
        );
        assert_eq!(package.build.as_ref().unwrap().steps[1], "make -j$(nproc)");
        assert_eq!(package.build.as_ref().unwrap().steps[2], "make install");
    }
}
