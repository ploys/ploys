use std::fmt::{self, Display};

/// The client error.
#[derive(Debug)]
pub enum Error {
    /// A request error.
    Request(reqwest::Error),
    /// A project error.
    Project(crate::project::Error<crate::repository::types::github::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(err) => Display::fmt(err, f),
            Self::Project(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}

impl From<crate::project::Error<crate::repository::types::github::Error>> for Error {
    fn from(err: crate::project::Error<crate::repository::types::github::Error>) -> Self {
        Self::Project(err)
    }
}
