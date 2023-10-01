use std::fmt::{self, Display};

/// The project error.
#[derive(Debug)]
pub enum Error {
    /// The Git source error.
    Git(crate::project::source::git::Error),
    /// The GitHub source error.
    GitHub(crate::project::source::github::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Git(git) => Display::fmt(git, f),
            Error::GitHub(github) => Display::fmt(github, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::project::source::git::Error> for Error {
    fn from(error: crate::project::source::git::Error) -> Self {
        Self::Git(error)
    }
}

impl From<crate::project::source::github::Error> for Error {
    fn from(error: crate::project::source::github::Error) -> Self {
        Self::GitHub(error)
    }
}
