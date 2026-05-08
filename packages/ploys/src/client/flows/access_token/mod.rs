mod error;

use reqwest::blocking::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::client::credentials::ServAddr;
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
        http_client: &HttpClient,
    ) -> Result<(), Self::Error> {
        match self.token.token_type() {
            TokenType::Personal | TokenType::OAuth | TokenType::User => {
                let user = http_client
                    .get("https://api.github.com/user")
                    .header("Accept", "application/vnd.github+json")
                    .header("X-GitHub-Api-Version", "2026-03-10")
                    .bearer_auth(self.token.value())
                    .send()?
                    .error_for_status()?
                    .json::<UserResponse>()?
                    .login;

                *credentials = Some(Credentials::new(
                    ServAddr::default(),
                    user,
                    self.token.clone(),
                ));

                Ok(())
            }
            TokenType::Installation => {
                let user = http_client
                    .post("https://api.github.com/graphql")
                    .bearer_auth(self.token.value())
                    .json(&GraphQLPayload {
                        query: "query { viewer { login } }",
                    })
                    .send()?
                    .error_for_status()?
                    .json::<InstallationResponse>()?
                    .data
                    .viewer
                    .login;

                *credentials = Some(Credentials::new(
                    ServAddr::default(),
                    user,
                    self.token.clone(),
                ));

                Ok(())
            }
            ty @ TokenType::Refresh => Err(Error::UnsupportedTokenType(ty)),
        }
    }
}

#[derive(Deserialize)]
struct UserResponse {
    login: String,
}

#[derive(Serialize)]
struct GraphQLPayload {
    query: &'static str,
}

#[derive(Deserialize)]
struct InstallationResponse {
    data: InstallationResponseData,
}

#[derive(Deserialize)]
struct InstallationResponseData {
    viewer: InstallationResponseViewer,
}

#[derive(Deserialize)]
struct InstallationResponseViewer {
    login: String,
}
