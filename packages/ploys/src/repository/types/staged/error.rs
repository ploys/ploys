use std::fmt::{self, Display};

/// The `Staged` repository error.
#[derive(Debug, PartialEq, Eq)]
pub enum Error<T> {
    /// An invalid path error.
    Path(crate::repository::path::Error),
    /// An inner repository error.
    Repo(T),
}

impl<T> Error<T>
where
    T: From<crate::repository::path::Error>,
{
    pub(crate) fn into_repo_err(self) -> T {
        match self {
            Error::Path(err) => err.into(),
            Error::Repo(err) => err,
        }
    }
}

impl<T> Display for Error<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(err) => Display::fmt(err, f),
            Self::Repo(err) => Display::fmt(err, f),
        }
    }
}

impl<T> std::error::Error for Error<T> where T: std::error::Error {}

impl<T> From<crate::repository::path::Error> for Error<T> {
    fn from(err: crate::repository::path::Error) -> Self {
        Self::Path(err)
    }
}
