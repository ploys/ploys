use std::fmt::{self, Display};

/// The device code authentication flow error.
#[derive(Debug)]
pub enum Error {
    /// A timeout error.
    Timeout,
    /// A request error.
    Request(reqwest::Error),
    /// Any other error when polling for the token.
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(f, "Timed out waiting for user input"),
            Self::Request(err) => Display::fmt(err, f),
            Self::Other(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}
