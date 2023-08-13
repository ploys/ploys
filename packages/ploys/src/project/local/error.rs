use std::fmt::{self, Display};
use std::io;

/// The local project error.
#[derive(Debug)]
pub enum Error {
    /// A Git error.
    Git(Box<gix::open::Error>),
    /// A remote error.
    Remote(Box<gix::remote::find::existing::Error>),
    /// An I/O error.
    Io(io::Error),
}

impl Error {
    /// Creates a remote not found error.
    pub(super) fn remote_not_found() -> Self {
        Self::Remote(Box::new(gix::remote::find::existing::Error::NotFound {
            name: String::from("origin").into(),
        }))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git(error) => Display::fmt(error, f),
            Self::Remote(error) => Display::fmt(error, f),
            Self::Io(error) => Display::fmt(error, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<gix::open::Error> for Error {
    fn from(error: gix::open::Error) -> Self {
        Self::Git(Box::new(error))
    }
}

impl From<gix::remote::find::existing::Error> for Error {
    fn from(error: gix::remote::find::existing::Error) -> Self {
        Self::Remote(Box::new(error))
    }
}
