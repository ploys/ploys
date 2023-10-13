use std::fmt::{self, Display};

/// The project error.
#[derive(Debug)]
pub enum Error {
    /// The Git source error.
    Git(crate::project::source::git::Error),
    /// The GitHub source error.
    GitHub(crate::project::source::github::Error),
    /// The package error.
    Package(crate::package::Error),
    /// The package bump error.
    Bump(crate::package::BumpError),
    /// The lockfile error.
    LockFile(crate::lockfile::Error),
    /// The package not found error.
    PackageNotFound(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git(git) => Display::fmt(git, f),
            Self::GitHub(github) => Display::fmt(github, f),
            Self::Package(err) => Display::fmt(err, f),
            Self::Bump(err) => Display::fmt(err, f),
            Self::LockFile(err) => Display::fmt(err, f),
            Self::PackageNotFound(name) => write!(f, "Package not found: `{name}`."),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::project::source::git::Error> for Error {
    fn from(err: crate::project::source::git::Error) -> Self {
        Self::Git(err)
    }
}

impl From<crate::project::source::github::Error> for Error {
    fn from(err: crate::project::source::github::Error) -> Self {
        Self::GitHub(err)
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
