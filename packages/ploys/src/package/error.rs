use std::convert::Infallible;
use std::fmt::{self, Display};

/// The package error.
#[derive(Debug)]
pub enum Error<T> {
    /// The repository error.
    Repository(T),
    /// A package manifest error.
    Manifest(super::manifest::Error),
    /// A package lockfile error.
    Lockfile(super::lockfile::Error),
    /// A package bump error.
    Bump(super::bump::Error),
    /// A UTF-8 error.
    Utf8(std::str::Utf8Error),
    /// A package not found error.
    NotFound(String),
}

impl<T> Display for Error<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(err) => Display::fmt(err, f),
            Self::Manifest(err) => Display::fmt(err, f),
            Self::Lockfile(err) => Display::fmt(err, f),
            Self::Bump(err) => Display::fmt(err, f),
            Self::Utf8(err) => Display::fmt(err, f),
            Self::NotFound(name) => write!(f, "Package not found: `{name}`."),
        }
    }
}

impl<T> std::error::Error for Error<T>
where
    T: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Repository(err) => Some(err),
            Self::Manifest(err) => Some(err),
            Self::Lockfile(err) => Some(err),
            Self::Bump(err) => Some(err),
            Self::Utf8(err) => Some(err),
            Self::NotFound(_) => None,
        }
    }
}

impl<T> From<Infallible> for Error<T> {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl<T> From<super::manifest::Error> for Error<T> {
    fn from(err: super::manifest::Error) -> Self {
        Self::Manifest(err)
    }
}

impl<T> From<super::lockfile::Error> for Error<T> {
    fn from(err: super::lockfile::Error) -> Self {
        Self::Lockfile(err)
    }
}

impl<T> From<super::bump::Error> for Error<T> {
    fn from(err: super::bump::Error) -> Self {
        Self::Bump(err)
    }
}

#[cfg(feature = "fs")]
impl From<std::io::Error> for Error<std::io::Error> {
    fn from(err: std::io::Error) -> Self {
        Self::Repository(err)
    }
}

#[cfg(feature = "git")]
impl From<crate::repository::git::Error> for Error<crate::repository::git::Error> {
    fn from(err: crate::repository::git::Error) -> Self {
        Self::Repository(err)
    }
}

#[cfg(feature = "github")]
impl From<crate::repository::github::Error> for Error<crate::repository::github::Error> {
    fn from(err: crate::repository::github::Error) -> Self {
        Self::Repository(err)
    }
}
