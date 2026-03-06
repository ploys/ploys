use std::fmt::{self, Debug, Display};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// An authentication token.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    value: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "time::serde::iso8601::option"
    )]
    expiry: Option<OffsetDateTime>,
}

impl Token {
    /// Constructs a new authentication token.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            expiry: None,
        }
    }

    /// Builds the token with the given expiry.
    pub fn with_expiry(mut self, expiry: impl Into<OffsetDateTime>) -> Self {
        self.set_expiry(expiry);
        self
    }
}

impl Token {
    /// Gets the token value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the token value.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }

    /// Gets the token expiry.
    pub fn expiry(&self) -> Option<&OffsetDateTime> {
        self.expiry.as_ref()
    }

    /// Sets the token expiry.
    pub fn set_expiry(&mut self, expiry: impl Into<OffsetDateTime>) {
        self.expiry = Some(expiry.into());
    }

    /// Checks if the token is expired.
    pub fn is_expired(&self) -> bool {
        match self.expiry {
            Some(exp) => exp <= OffsetDateTime::now_utc(),
            None => false,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("value", &"***")
            .field("expiry", &self.expiry)
            .finish()
    }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_string(),
            expiry: None,
        }
    }
}

impl From<String> for Token {
    fn from(value: String) -> Self {
        Self {
            value,
            expiry: None,
        }
    }
}
