use std::fmt::{self, Display};

/// The token error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Empty,
    Invalid,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty token"),
            Self::Invalid => write!(f, "Invalid token"),
        }
    }
}

impl std::error::Error for Error {}
