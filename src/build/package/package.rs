use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,

    pub source: Source,
    pub dependencies: Dependencies,
    pub build: Option<Build>,
    pub install_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub install_path: String,
    pub dependencies: Vec<Dependency>,
}

// TODO
// support other sources
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Source {
    Git {
        repo: String,
        tag: Option<String>,
        branch: Option<String>,
        commit: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Build {
    pub steps: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependencies {
    pub runtime: Vec<Dependency>,
    pub build: Vec<Dependency>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version_req: Option<String>,
}
