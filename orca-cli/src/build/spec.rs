use semver::{Version, VersionReq};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct DependencyDeclaration {
    pub name: String,
    pub version: VersionReq,
}

#[derive(Deserialize)]
pub struct BuildSpec {
    pub name: String,
    pub version: Version,
    #[serde(default)]
    pub dependencies: Vec<DependencyDeclaration>,
}
