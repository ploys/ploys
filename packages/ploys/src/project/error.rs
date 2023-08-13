use std::fmt::{self, Display};

/// The project error.
#[derive(Debug)]
pub enum Error {
    /// The local project error.
    Local(super::local::Error),
    /// The remote project error.
    Remote(super::remote::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Local(local) => Display::fmt(local, f),
            Error::Remote(remote) => Display::fmt(remote, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<super::local::Error> for Error {
    fn from(error: super::local::Error) -> Self {
        Self::Local(error)
    }
}

impl From<super::remote::Error> for Error {
    fn from(error: super::remote::Error) -> Self {
        Self::Remote(error)
    }
}
