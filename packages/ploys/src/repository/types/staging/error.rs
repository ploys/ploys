use std::fmt::{self, Display};

/// The `Staging` repository error.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// An invalid path error.
    Path(crate::repository::path::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::repository::path::Error> for Error {
    fn from(err: crate::repository::path::Error) -> Self {
        Self::Path(err)
    }
}
