use std::fmt::{self, Display};

/// The project error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The repository error.
    Repository(crate::repository::Error),
    /// The package error.
    Package(crate::package::Error),
    /// The package bump error.
    Bump(crate::package::BumpError),
    /// The lockfile error.
    LockFile(crate::lockfile::Error),
    /// The package not found error.
    PackageNotFound(String),
    /// The action is not supported.
    Unsupported,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(err) => Display::fmt(err, f),
            Self::Package(err) => Display::fmt(err, f),
            Self::Bump(err) => Display::fmt(err, f),
            Self::LockFile(err) => Display::fmt(err, f),
            Self::PackageNotFound(name) => write!(f, "Package not found: `{name}`."),
            Self::Unsupported => write!(f, "Action not supported"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::repository::Error> for Error {
    fn from(err: crate::repository::Error) -> Self {
        Self::Repository(err)
    }
}

impl From<crate::package::Error> for Error {
    fn from(err: crate::package::Error) -> Self {
        Self::Package(err)
    }
}

impl From<crate::package::BumpError> for Error {
    fn from(err: crate::package::BumpError) -> Self {
        Self::Bump(err)
    }
}

impl From<crate::lockfile::Error> for Error {
    fn from(err: crate::lockfile::Error) -> Self {
        Self::LockFile(err)
    }
}

#[cfg(feature = "git")]
impl From<crate::repository::git::Error> for Error {
    fn from(err: crate::repository::git::Error) -> Self {
        Self::Repository(err.into())
    }
}

#[cfg(feature = "github")]
impl From<crate::repository::github::Error> for Error {
    fn from(err: crate::repository::github::Error) -> Self {
        Self::Repository(err.into())
    }
}
