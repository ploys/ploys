use std::fmt::{self, Display};

/// The refresh token authentication flow adapter error.
#[derive(Debug)]
pub enum Error<T> {
    /// A request error.
    Request(reqwest::Error),
    /// An error with the adapted authentication flow.
    Inner(T),
}

impl<T> Display for Error<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(err) => Display::fmt(err, f),
            Self::Inner(err) => Display::fmt(err, f),
        }
    }
}

impl<T> std::error::Error for Error<T> where T: std::error::Error {}

impl<T> From<reqwest::Error> for Error<T> {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}
