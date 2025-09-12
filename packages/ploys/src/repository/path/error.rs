use std::fmt::{self, Display};

use relative_path::RelativePathBuf;

/// An invalid path error.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// An empty path.
    Empty,
    /// A path that escapes the repository.
    Escape(RelativePathBuf),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Path is empty"),
            Self::Escape(path) => write!(f, "Path escapes repository: `{path}`"),
        }
    }
}

impl std::error::Error for Error {}
