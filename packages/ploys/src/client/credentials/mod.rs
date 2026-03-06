mod token;

pub use self::token::Token;

/// The client authentication credentials.
#[derive(Clone, Debug, Default)]
pub struct Credentials {
    access_token: Option<Token>,
}

impl Credentials {
    /// Constructs new client authentication credentials.
    pub fn new() -> Self {
        Self { access_token: None }
    }

    /// Builds the credentials with the given access token.
    pub fn with_access_token(mut self, token: impl Into<Token>) -> Self {
        self.set_access_token(token);
        self
    }
}

impl Credentials {
    /// Gets the access token.
    pub fn get_access_token(&self) -> Option<Token> {
        self.access_token.clone()
    }

    /// Sets the access token.
    pub fn set_access_token(&mut self, token: impl Into<Token>) {
        self.access_token = Some(token.into());
    }
}
