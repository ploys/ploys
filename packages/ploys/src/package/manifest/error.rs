use std::fmt::{self, Display};

use crate::file::ParseError;

/// A package manifest error.
#[derive(Debug)]
pub enum Error {
    /// A glob error.
    Glob(globset::Error),
    /// A parse error.
    Parse(ParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glob(err) => Display::fmt(err, f),
            Self::Parse(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Glob(err) => Some(err),
            Self::Parse(err) => Some(err),
        }
    }
}

impl From<globset::Error> for Error {
    fn from(err: globset::Error) -> Self {
        Self::Glob(err)
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
