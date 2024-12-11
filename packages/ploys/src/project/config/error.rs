use std::fmt::{self, Display};

use crate::file::ParseError;

/// The configuration error.
#[derive(Debug)]
pub enum Error {
    /// A missing configuration error.
    Missing,
    /// An invalid configuration error.
    Invalid,
    /// A parse error.
    Parse(ParseError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Missing | Self::Invalid => None,
            Self::Parse(err) => Some(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "Missing project configuration"),
            Self::Invalid => write!(f, "Invalid project configuration"),
            Self::Parse(err) => Display::fmt(err, f),
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl From<toml_edit::TomlError> for Error {
    fn from(err: toml_edit::TomlError) -> Self {
        Self::Parse(err.into())
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Parse(err.into())
    }
}
