use std::fmt::{self, Display};
use std::io;

/// The `FileSystem` repository error.
#[derive(Debug)]
pub enum Error {
    /// An I/O error.
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Self {
        Self::Io(err.into())
    }
}
