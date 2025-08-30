use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

/// The `FileSystem` repository error.
#[derive(Debug)]
pub enum Error {
    /// An invalid directory error.
    Directory(PathBuf),
    /// An I/O error.
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory(path) => write!(f, "Invalid directory: `{}`", path.display()),
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

impl From<crate::repository::staged::Error<Error>> for Error {
    fn from(err: crate::repository::staged::Error<Error>) -> Self {
        match err {
            crate::repository::staged::Error::Inner(err) => err,
        }
    }
}
