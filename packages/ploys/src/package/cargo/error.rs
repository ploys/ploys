use std::fmt::{self, Display};

/// A cargo package error.
#[derive(Debug)]
pub enum Error {
    /// A glob error.
    Glob(globset::Error),
    /// A manifest error.
    Manifest(toml_edit::TomlError),
    /// A UTF-8 error.
    Utf8(std::str::Utf8Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glob(err) => Display::fmt(err, f),
            Self::Manifest(err) => Display::fmt(err, f),
            Self::Utf8(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<globset::Error> for Error {
    fn from(err: globset::Error) -> Self {
        Self::Glob(err)
    }
}

impl From<toml_edit::TomlError> for Error {
    fn from(err: toml_edit::TomlError) -> Self {
        Self::Manifest(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}
