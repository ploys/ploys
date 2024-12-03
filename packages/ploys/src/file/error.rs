use std::fmt::{self, Display};

/// A file parse error.
#[derive(Debug)]
pub enum ParseError {
    /// A TOML error.
    Toml(toml_edit::TomlError),
    /// A UTF-8 error.
    Utf8(std::str::Utf8Error),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Toml(err) => Display::fmt(err, f),
            Self::Utf8(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Toml(err) => Some(err),
            Self::Utf8(err) => Some(err),
        }
    }
}

impl From<toml_edit::TomlError> for ParseError {
    fn from(err: toml_edit::TomlError) -> Self {
        Self::Toml(err)
    }
}

impl From<std::str::Utf8Error> for ParseError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}
