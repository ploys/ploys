use std::fmt::{self, Display};

/// The lockfile error.
#[derive(Debug)]
pub enum Error {
    /// A cargo lockfile error.
    Cargo(super::cargo::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cargo(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<super::cargo::Error> for Error {
    fn from(err: super::cargo::Error) -> Self {
        Self::Cargo(err)
    }
}
