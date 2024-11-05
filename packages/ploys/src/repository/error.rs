use std::fmt::{self, Display};

/// The project source error.
#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "git")]
    Git(super::git::Error),
    #[cfg(feature = "github")]
    GitHub(super::github::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "git")]
            Self::Git(git) => Some(git),
            #[cfg(feature = "github")]
            Self::GitHub(github) => Some(github),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "git")]
            Self::Git(git) => Display::fmt(git, f),
            #[cfg(feature = "github")]
            Self::GitHub(github) => Display::fmt(github, f),
        }
    }
}

#[cfg(feature = "git")]
impl From<super::git::Error> for Error {
    fn from(err: super::git::Error) -> Self {
        Self::Git(err)
    }
}

#[cfg(feature = "github")]
impl From<super::github::Error> for Error {
    fn from(err: super::github::Error) -> Self {
        Self::GitHub(err)
    }
}
