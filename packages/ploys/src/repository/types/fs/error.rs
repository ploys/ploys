use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

/// The `FileSystem` repository error.
#[derive(Debug)]
pub enum Error {
    /// An invalid directory error.
    Directory(PathBuf),
    /// An invalid path error.
    Path(crate::repository::path::Error),
    /// An I/O error.
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory(path) => write!(f, "Invalid directory: `{}`", path.display()),
            Self::Path(err) => Display::fmt(err, f),
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
