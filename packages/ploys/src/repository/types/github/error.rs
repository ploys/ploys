use std::convert::Infallible;
use std::fmt::{self, Display};
use std::io;

use crate::repository::RepoAddrError;

/// The GitHub repository error.
#[derive(Debug)]
pub enum Error {
    /// An invalid path error.
    Path(crate::repository::path::Error),
    /// A request error.
    Request(reqwest::Error),
    /// An I/O error.
    Io(io::Error),
    /// An address error.
    Addr(RepoAddrError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(err) => Display::fmt(err, f),
            Self::Request(transport) => Display::fmt(transport, f),
            Self::Io(err) => Display::fmt(err, f),
            Self::Addr(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Path(err) => Some(err),
            Self::Request(err) => Some(err),
            Self::Io(err) => Some(err),
            Self::Addr(err) => Some(err),
        }
    }
}

impl From<crate::repository::path::Error> for Error {
    fn from(err: crate::repository::path::Error) -> Self {
        Self::Path(err)
    }
}

impl From<RepoAddrError> for Error {
    fn from(err: RepoAddrError) -> Self {
        Self::Addr(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}
