mod server;
mod token;

pub use self::server::ServAddr;
pub use self::token::{Error as TokenError, Token, TokenType};

/// The client authentication credentials.
#[derive(Clone, Debug)]
pub struct Credentials {
    server: ServAddr,
    user: String,
    access_token: Token,
}

impl Credentials {
    /// Constructs new client authentication credentials.
    pub(crate) fn new(
        server: impl Into<ServAddr>,
        user: impl Into<String>,
        access_token: impl Into<Token>,
    ) -> Self {
        Self {
            server: server.into(),
            user: user.into(),
            access_token: access_token.into(),
        }
    }
}

impl Credentials {
    /// Gets the server address.
    pub fn server(&self) -> &ServAddr {
        &self.server
    }

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

    /// Checks if the credentials have expired.
    pub fn is_expired(&self) -> bool {
        self.access_token.is_expired()
    }
}
