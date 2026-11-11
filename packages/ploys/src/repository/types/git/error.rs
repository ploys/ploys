use std::convert::Infallible;
use std::fmt::{self, Display};
use std::io;

/// The Git repository error.
#[derive(Debug)]
pub enum Error {
    /// An invalid path error.
    Path(crate::repository::path::Error),
    /// A `gix` error.
    Gix(gix::Error),
    /// An I/O error.
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(err) => Display::fmt(err, f),
            Self::Gix(err) => Display::fmt(err, f),
            Self::Io(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::repository::path::Error> for Error {
    fn from(err: crate::repository::path::Error) -> Self {
        Self::Path(err)
    }
}

impl From<gix::error::Error> for Error {
    fn from(err: gix::error::Error) -> Self {
        Self::Gix(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}
