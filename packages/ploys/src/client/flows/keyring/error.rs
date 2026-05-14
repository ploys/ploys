use std::fmt::{self, Display};

/// The keyring authentication flow adapter error.
#[derive(Debug)]
pub enum Error<T> {
    /// A keyring error.
    Keyring(keyring_core::Error),
    /// A prompt error.
    Prompt(dialoguer::Error),
    /// A JSON de/serialization error.
    Json(serde_json::Error),
    /// An error with the adapted authentication flow.
    Inner(T),
}

impl<T> Display for Error<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyring(err) => Display::fmt(err, f),
            Self::Prompt(err) => Display::fmt(err, f),
            Self::Json(err) => Display::fmt(err, f),
            Self::Inner(err) => Display::fmt(err, f),
        }
    }
}

impl<T> std::error::Error for Error<T> where T: std::error::Error {}

impl<T> From<keyring_core::Error> for Error<T> {
    fn from(err: keyring_core::Error) -> Self {
        Self::Keyring(err)
    }
}

impl<T> From<dialoguer::Error> for Error<T> {
    fn from(err: dialoguer::Error) -> Self {
        Self::Prompt(err)
    }
}

impl<T> From<serde_json::Error> for Error<T> {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}
