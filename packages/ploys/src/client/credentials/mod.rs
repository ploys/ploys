mod token;

pub use self::token::{Error as TokenError, Token, TokenType};

/// The client authentication credentials.
#[derive(Clone, Debug)]
pub struct Credentials {
    access_token: Token,
}

impl Credentials {
    /// Constructs new client authentication credentials.
    pub(crate) fn new(access_token: impl Into<Token>) -> Self {
        Self {
            access_token: access_token.into(),
        }
    }
}

impl Credentials {
    /// Gets the access token.
    pub fn access_token(&self) -> &Token {
        &self.access_token
    }

    /// Sets the access token.
    pub fn set_access_token(&mut self, token: impl Into<Token>) {
        self.access_token = token.into();
    }

    /// Checks if the credentials have expired.
    pub fn is_expired(&self) -> bool {
        self.access_token.is_expired()
    }
}
