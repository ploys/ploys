mod error;

use reqwest::blocking::Client as HttpClient;

use crate::client::{Credentials, Token, TokenType};

pub use self::error::Error;

use super::Authenticate;

/// The access token authentication flow.
///
/// This authentication flow uses the stored access token to obtain credentials
/// for an authenticated user or app installation.
#[derive(Clone, Debug)]
pub struct AccessTokenFlow {
    token: Token,
}

impl AccessTokenFlow {
    /// Constructs a new access token authentication flow.
    pub fn new(token: impl Into<Token>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl Authenticate for AccessTokenFlow {
    type Error = Error;

    fn authenticate(
        &self,
        credentials: &mut Option<Credentials>,
        _: &HttpClient,
    ) -> Result<(), Self::Error> {
        match self.token.token_type() {
            TokenType::Personal | TokenType::OAuth | TokenType::User | TokenType::Installation => {
                *credentials = Some(Credentials::new(self.token.clone()));

                Ok(())
            }
            ty @ TokenType::Refresh => Err(Error::UnsupportedTokenType(ty)),
        }
    }
}
