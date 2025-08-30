use std::fmt::{self, Display};

/// The `Staged` repository error.
#[derive(Debug, PartialEq, Eq)]
pub enum Error<T> {
    /// An inner repository error.
    Inner(T),
}

impl<T> Display for Error<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inner(err) => Display::fmt(err, f),
        }
    }
}

impl<T> std::error::Error for Error<T> where T: std::error::Error {}

impl<T> From<T> for Error<T> {
    fn from(err: T) -> Self {
        Self::Inner(err)
    }
}
