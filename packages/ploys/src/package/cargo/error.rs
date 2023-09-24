use std::fmt::{self, Display};

/// A cargo package error.
#[derive(Debug)]
pub enum Error {
    /// A glob error.
    Glob(globset::Error),
    /// A manifest error.
    Manifest(cargo_toml::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glob(err) => Display::fmt(err, f),
            Self::Manifest(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<globset::Error> for Error {
    fn from(err: globset::Error) -> Self {
        Self::Glob(err)
    }
}

impl From<cargo_toml::Error> for Error {
    fn from(err: cargo_toml::Error) -> Self {
        Self::Manifest(err)
    }
}
