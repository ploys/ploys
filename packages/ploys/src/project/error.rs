use std::convert::Infallible;
use std::fmt::{self, Display};

/// The project error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The configuration error.
    Config(super::config::Error),
    /// The changelog error.
    Changelog(crate::changelog::Error),
    /// The repository error.
    Repository(crate::repository::Error),
    /// The package error.
    Package(crate::package::Error),
    /// The action is not supported.
    Unsupported,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Changelog(err) => Display::fmt(err, f),
            Self::Config(err) => Display::fmt(err, f),
            Self::Repository(err) => Display::fmt(err, f),
            Self::Package(err) => Display::fmt(err, f),
            Self::Unsupported => write!(f, "Action not supported"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl From<super::config::Error> for Error {
    fn from(err: super::config::Error) -> Self {
        Self::Config(err)
    }
}

impl From<crate::changelog::Error> for Error {
    fn from(err: crate::changelog::Error) -> Self {
        Self::Changelog(err)
    }
}

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
        Self::Package(err.into())
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
