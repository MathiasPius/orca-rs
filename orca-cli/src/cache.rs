use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
};

use semver::{Version, VersionReq};

use crate::identifier::{self, Identifier, NameIdentifier, VersionIdentifier};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("identifier parsing error")]
    Identifier(#[from] identifier::ParsingError),
    #[error("unnamed package: {0}")]
    UnnamedPackage(PathBuf),
}

#[derive(Debug)]
pub struct Artifact {
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct Package {
    pub name: Vec<NameIdentifier>,
    pub version: VersionIdentifier,
    pub artifacts: Vec<Artifact>,
}

pub struct Cache {
    path: PathBuf,
}

impl Cache {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Cache {
            path: std::env::current_dir()?.join(path),
        })
    }

    pub fn list(&self) -> Result<Vec<Package>, Error> {
        let mut packages = Vec::new();
        println!("Listing all packages in {}", self.path.to_str().unwrap());
        walk(&mut packages, Vec::new(), &self.path)?;

        Ok(packages)
    }

    pub fn get(name: Identifier, version: Version) -> Option<Package> {
        None
    }

    pub fn list_versions(name: Identifier) -> Vec<Package> {
        vec![]
    }

    pub fn find(name: Identifier, required_version: VersionReq) -> Option<Package> {
        None
    }

    pub fn put(build: Package) {}
}

fn list_artifacts<P: AsRef<Path>>(path: P) -> Result<Vec<Artifact>, Error> {
    let entries: Result<Vec<_>, _> = std::fs::read_dir(path)?.into_iter().collect();

    Ok(entries?
        .into_iter()
        .map(|entry| Artifact { path: entry.path() })
        .collect())
}

// Recursively walks a path structure, looking for packages.
fn walk<P: AsRef<Path>>(
    packages: &mut Vec<Package>,
    package_name: Vec<NameIdentifier>,
    path: P,
) -> Result<Vec<(Version, Vec<Artifact>)>, Error> {
    let entries: Result<Vec<_>, _> = std::fs::read_dir(&path)?.into_iter().collect();

    println!("Identifier: {:?}", entries);

    for entry in entries? {
        let identifier = Identifier::try_from(entry.file_name().to_string_lossy().to_string())?;

        match identifier {
            Identifier::Name(name) => {
                let mut extended_name = package_name.clone();
                extended_name.push(name);
                walk(packages, extended_name, entry.path())?;
            }
            Identifier::Version(version) => {
                if package_name.is_empty() {
                    return Err(Error::UnnamedPackage(path.as_ref().to_path_buf()));
                }

                packages.push(Package {
                    name: package_name.clone(),
                    version,
                    artifacts: list_artifacts(&path)?,
                })
            }
        }
    }

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::Cache;

    #[test]
    fn test_cache_listing() {
        let cache = Cache::new(".orca/cache").unwrap();

        println!("{:#?}", cache.list().unwrap());
    }
}

