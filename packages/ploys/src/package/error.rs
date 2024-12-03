use std::fmt::{self, Display};

/// The package error.
#[derive(Debug)]
pub enum Error {
    /// A package manifest error.
    Manifest(super::manifest::Error),
    /// A package lockfile error.
    Lockfile(super::lockfile::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(err) => Display::fmt(err, f),
            Self::Lockfile(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Manifest(err) => Some(err),
            Self::Lockfile(err) => Some(err),
        }
    }
}

impl From<super::manifest::Error> for Error {
    fn from(err: super::manifest::Error) -> Self {
        Self::Manifest(err)
    }
}

impl From<super::lockfile::Error> for Error {
    fn from(err: super::lockfile::Error) -> Self {
        Self::Lockfile(err)
    }
}
