use std::convert::Infallible;
use std::fmt::{self, Display};

/// The project error.
#[derive(Debug)]
pub enum Error<T = Infallible> {
    /// The configuration error.
    Config(super::config::Error),
    /// The changelog error.
    Changelog(crate::changelog::Error),
    /// The repository error.
    Repository(T),
    /// The package error.
    Package(crate::package::Error<T>),
    /// A UTF-8 error.
    Utf8(std::str::Utf8Error),
}

impl<T> Display for Error<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Changelog(err) => Display::fmt(err, f),
            Self::Config(err) => Display::fmt(err, f),
            Self::Repository(err) => Display::fmt(err, f),
            Self::Package(err) => Display::fmt(err, f),
            Self::Utf8(err) => Display::fmt(err, f),
        }
    }
}

impl<T> std::error::Error for Error<T>
where
    T: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Changelog(err) => Some(err),
            Self::Config(err) => Some(err),
            Self::Repository(err) => Some(err),
            Self::Package(err) => Some(err),
            Self::Utf8(err) => Some(err),
        }
    }
}

impl<T> From<Infallible> for Error<T> {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl<T> From<super::config::Error> for Error<T> {
    fn from(err: super::config::Error) -> Self {
        Self::Config(err)
    }
}

impl<T> From<crate::changelog::Error> for Error<T> {
    fn from(err: crate::changelog::Error) -> Self {
        Self::Changelog(err)
    }
}

impl<T> From<crate::package::Error<T>> for Error<T> {
    fn from(err: crate::package::Error<T>) -> Self {
        Self::Package(err)
    }
}

impl<T> From<crate::package::BumpError> for Error<T> {
    fn from(err: crate::package::BumpError) -> Self {
        Self::Package(err.into())
    }
}

#[cfg(feature = "fs")]
impl From<crate::repository::types::fs::Error> for Error<crate::repository::types::fs::Error> {
    fn from(err: crate::repository::types::fs::Error) -> Self {
        Self::Repository(err)
    }
}

#[cfg(feature = "git")]
impl From<crate::repository::types::git::Error> for Error<crate::repository::types::git::Error> {
    fn from(err: crate::repository::types::git::Error) -> Self {
        Self::Repository(err)
    }
}

#[cfg(feature = "github")]
impl From<crate::repository::types::github::Error>
    for Error<crate::repository::types::github::Error>
{
    fn from(err: crate::repository::types::github::Error) -> Self {
        Self::Repository(err)
    }
}
