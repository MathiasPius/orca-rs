use semver::VersionReq;


#[derive(Deserialize)]
pub struct DependencyDeclaration {
    pub name: String,
    pub version: VersionReq,
}

#[derive(Deserialize)]
pub struct BuildSpec {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Vec<DependencyDeclaration>
}