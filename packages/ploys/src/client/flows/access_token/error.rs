use std::fmt::{self, Display};

use crate::client::TokenType;

/// The access token authentication flow error.
#[derive(Debug)]
pub enum Error {
    /// An unsupported token type.
    UnsupportedTokenType(TokenType),
    /// A request error.
    Request(reqwest::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTokenType(ty) => write!(f, "Unsupported token type: {ty}"),
            Self::Request(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}
