use crate::build::package::package::Package;
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
