mod error;

use std::fmt::{self, Debug, Display, from_fn};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use time::OffsetDateTime;

pub use self::error::Error;

/// An authentication token.
#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    #[serde_as(as = "DisplayFromStr")]
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
    pub fn new(value: impl AsRef<str>) -> Result<Self, Error> {
        value.as_ref().parse()
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

    /// Gets the type of authentication token.
    ///
    /// GitHub use identifiable prefixes on all authentication tokens as of
    /// April 2021. The [announcement][1] and subsequent [blog post][2] provide
    /// further information about the format.
    ///
    /// [1]: https://github.blog/changelog/2021-03-31-authentication-token-format-updates-are-generally-available/
    /// [2]: https://github.blog/engineering/platform-security/behind-githubs-new-authentication-token-formats/
    pub fn token_type(&self) -> TokenType {
        match &self.value[..4] {
            "ghp_" => TokenType::Personal,
            "gho_" => TokenType::OAuth,
            "ghu_" => TokenType::User,
            "ghs_" => TokenType::Installation,
            "ghr_" => TokenType::Refresh,
            _ => unreachable!("construction via `FromStr` impl"),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match f.alternate() {
            true => write!(f, "{}***", &self.value[..4]),
            false => Display::fmt(&self.value, f),
        }
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("value", &from_fn(|f| write!(f, "{}***", &self.value[..4])))
            .field("expiry", &self.expiry)
            .finish()
    }
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Err(Error::Empty);
        }

        match value.split_once('_') {
            Some(("ghp" | "gho" | "ghu" | "ghs" | "ghr", key)) if !key.is_empty() => Ok(Self {
                value: value.to_string(),
                expiry: None,
            }),
            _ => Err(Error::Invalid),
        }
    }
}

impl TryFrom<&str> for Token {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

/// The type of authentication token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TokenType {
    /// A personal access token.
    Personal,
    /// An OAuth access token.
    OAuth,
    /// A GitHub App user-to-server (on behalf of) token.
    User,
    /// A GitHub App server-to-server (installation) token.
    Installation,
    /// A GitHub App user-to-server refresh token.
    Refresh,
}

#[cfg(test)]
mod tests {
    use super::{Error, Token, TokenType};

    #[test]
    fn test_token_type() {
        let ghp = Token::new("ghp_abc_DEF_123").unwrap();
        let gho = Token::new("gho_abc_DEF_123").unwrap();
        let ghu = Token::new("ghu_abc_DEF_123").unwrap();
        let ghs = Token::new("ghs_abc_DEF_123").unwrap();
        let ghr = Token::new("ghr_abc_DEF_123").unwrap();

        assert_eq!(ghp.token_type(), TokenType::Personal);
        assert_eq!(gho.token_type(), TokenType::OAuth);
        assert_eq!(ghu.token_type(), TokenType::User);
        assert_eq!(ghs.token_type(), TokenType::Installation);
        assert_eq!(ghr.token_type(), TokenType::Refresh);
    }

    #[test]
    fn test_token_validation() {
        assert_eq!(Token::new(""), Err(Error::Empty));
        assert_eq!(Token::new("gho"), Err(Error::Invalid));
        assert_eq!(Token::new("gho_"), Err(Error::Invalid));
        assert_eq!(Token::new("ghx_abc_DEF_123"), Err(Error::Invalid));
    }
}
