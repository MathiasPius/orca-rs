use std::convert::TryFrom;

use semver::Version;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("Identifier starts with number, but is not a valid SemVer string")]
    InvalidVersion(#[from] semver::Error),
    #[error("zero-length string is not a valid identifier")]
    ZeroLength,
}

pub type NameIdentifier = String;
pub type VersionIdentifier = Version;

#[derive(Debug)]
pub enum Identifier {
    Name(NameIdentifier),
    Version(VersionIdentifier),
}

impl TryFrom<String> for Identifier {
    type Error = ParsingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let first_character = value.chars().next().ok_or(ParsingError::ZeroLength)?;

        Ok(if first_character.is_digit(10) {
            Identifier::Version(VersionIdentifier::parse(&value)?)
        } else {
            Identifier::Name(value)
        })
    }
}
