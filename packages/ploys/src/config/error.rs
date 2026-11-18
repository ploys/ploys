use std::fmt::{self, Display};

/// The configuration error.
#[derive(Debug)]
pub enum Error {
    /// A missing configuration error.
    Missing,
    /// An invalid configuration error.
    Invalid,
    /// A TOML error.
    Toml(toml_edit::TomlError),
    /// A UTF-8 error.
    Utf8(std::str::Utf8Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Missing | Self::Invalid => None,
            Self::Toml(err) => Some(err),
            Self::Utf8(err) => Some(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "Missing project configuration"),
            Self::Invalid => write!(f, "Invalid project configuration"),
            Self::Toml(err) => Display::fmt(err, f),
            Self::Utf8(err) => Display::fmt(err, f),
        }
    }
}

impl From<toml_edit::TomlError> for Error {
    fn from(err: toml_edit::TomlError) -> Self {
        Self::Toml(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}
