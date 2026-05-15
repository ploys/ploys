mod error;

use std::sync::Arc;

use once_cell::sync::OnceCell;
use reqwest::blocking::Client as HttpClient;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use time::OffsetDateTime;

use crate::client::{Credentials, ServAddr, Token};

pub use self::error::Error;

use super::Authenticate;

/// The refresh token authentication flow adapter.
///
/// Note that this currently only supports refreshing credentials created via
/// the device code flow.
#[derive(Clone, Debug, Default)]
pub struct RefreshTokenFlow<T> {
    client_id: OnceCell<Arc<str>>,
    auth_flow: T,
}

impl<T> RefreshTokenFlow<T> {
    /// Constructs a new refresh token authentication flow adapter.
    pub fn new(auth_flow: T) -> Self {
        Self {
            client_id: OnceCell::new(),
            auth_flow,
        }
    }
}

impl<T> Authenticate for RefreshTokenFlow<T>
where
    T: Authenticate,
{
    type Error = Error<T::Error>;

    fn authenticate(
        &self,
        credentials: &mut Option<Credentials>,
        http_client: &HttpClient,
        server: &ServAddr,
    ) -> Result<(), Self::Error> {
        let refresh_token = match credentials.as_ref().and_then(Credentials::refresh_token) {
            Some(refresh_token) if refresh_token.is_expired() => None,
            Some(refresh_token) => Some(refresh_token),
            None => None,
        };

        let Some(refresh_token) = refresh_token else {
            self.auth_flow
                .authenticate(credentials, http_client, server)
                .map_err(Error::Inner)?;

            return Ok(());
        };

        let client_id = self.client_id.get_or_try_init(|| {
            Ok::<_, Error<T::Error>>(
                http_client
                    .get(format!("https://{server}/github"))
                    .send()?
                    .error_for_status()?
                    .json::<AppInfo>()?
                    .client_id
                    .into(),
            )
        })?;

        let token_response: TokenResponse = http_client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", &**client_id),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.value()),
            ])
            .send()?
            .error_for_status()?
            .json()?;

        let now = OffsetDateTime::now_utc();
        let mut access_token = token_response.access_token;

        if let Some(expires_in) = token_response.expires_in {
            access_token.set_expiry(now + time::Duration::seconds(expires_in));
        }

        let user = http_client
            .get("https://api.github.com/user")
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .bearer_auth(access_token.value())
            .send()?
            .error_for_status()?
            .json::<UserResponse>()?
            .login;

        let mut creds = Credentials::new(user, access_token);

        if let Some(mut refresh_token) = token_response.refresh_token {
            if let Some(expires_in) = token_response.refresh_token_expires_in {
                refresh_token.set_expiry(now + time::Duration::seconds(expires_in));
            }

            creds.set_refresh_token(refresh_token);
        }

        *credentials = Some(creds);

        Ok(())
    }
}

#[derive(Deserialize)]
struct AppInfo {
    client_id: String,
}

#[serde_as]
#[derive(Deserialize)]
struct TokenResponse {
    #[serde_as(as = "DisplayFromStr")]
    access_token: Token,
    expires_in: Option<i64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    refresh_token: Option<Token>,
    refresh_token_expires_in: Option<i64>,
}

#[derive(Deserialize)]
struct UserResponse {
    login: String,
}
