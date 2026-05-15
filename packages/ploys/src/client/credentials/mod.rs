mod token;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub use self::token::{Error as TokenError, Token, TokenType};

/// The client authentication credentials.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credentials {
    user: String,
    access_token: Token,
    refresh_token: Option<Token>,
}

impl Credentials {
    /// Constructs new client authentication credentials.
    pub(crate) fn new(user: impl Into<String>, access_token: impl Into<Token>) -> Self {
        Self {
            user: user.into(),
            access_token: access_token.into(),
            refresh_token: None,
        }
    }
}

impl Credentials {
    /// Gets the user login name.
    ///
    /// Note that for an app installation this would contain the `[bot]` suffix
    /// to differentiate between users and apps.
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Gets the access token.
    pub fn access_token(&self) -> &Token {
        &self.access_token
    }

    /// Sets the access token.
    pub fn set_access_token(&mut self, token: impl Into<Token>) {
        self.access_token = token.into();
    }

    /// Gets the refresh token.
    pub fn refresh_token(&self) -> Option<&Token> {
        self.refresh_token.as_ref()
    }

    /// Sets the refresh token.
    pub fn set_refresh_token(&mut self, token: impl Into<Token>) {
        self.refresh_token = Some(token.into());
    }

    /// Builds the credentials with the given refresh token.
    pub fn with_refresh_token(mut self, token: impl Into<Token>) -> Self {
        self.set_refresh_token(token);
        self
    }

    /// Gets the credentials expiry.
    pub fn expiry(&self) -> Option<&OffsetDateTime> {
        match self.refresh_token() {
            Some(refresh_token) => refresh_token.expiry(),
            None => self.access_token.expiry(),
        }
    }

    /// Checks if the credentials have expired.
    pub fn is_expired(&self) -> bool {
        match self.refresh_token() {
            Some(refresh_token) => refresh_token.is_expired(),
            None => self.access_token.is_expired(),
        }
    }
}
