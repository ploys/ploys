use std::fmt::{self, Display};

/// A repository address error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// An invalid repository address.
    Invalid(String),
}

impl Error {
    /// Constructs a new invalid error variant.
    pub fn invalid(input: impl Into<String>) -> Self {
        Self::Invalid(input.into())
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(input) => write!(f, "Invalid repository address: {input}"),
        }
    }
}
