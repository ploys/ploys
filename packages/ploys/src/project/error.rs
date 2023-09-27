use std::fmt::{self, Display};

/// The project error.
#[derive(Debug)]
pub enum Error {
    /// The Git project error.
    Git(super::git::Error),
    /// The GitHub project error.
    GitHub(super::github::Error),
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

impl From<super::git::Error> for Error {
    fn from(error: super::git::Error) -> Self {
        Self::Git(error)
    }
}

impl From<super::github::Error> for Error {
    fn from(error: super::github::Error) -> Self {
        Self::GitHub(error)
    }
}
