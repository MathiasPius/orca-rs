use semver::{Version, VersionReq};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: VersionReq,
}

#[derive(Debug, Deserialize)]
pub struct BuildSpec {
    pub name: String,
    pub version: Version,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}
