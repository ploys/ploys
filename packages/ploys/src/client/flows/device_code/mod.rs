mod error;

use std::thread::sleep;
use std::time::{Duration, Instant};

use reqwest::blocking::Client as HttpClient;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use time::OffsetDateTime;
use url::Url;

use crate::client::{Credentials, ServAddr, Token};

pub use self::error::Error;

use super::Authenticate;

/// The device code authentication flow.
#[derive(Clone, Debug, Default)]
pub struct DeviceCodeFlow;

impl DeviceCodeFlow {
    /// Constructs a new device code authentication flow.
    pub fn new() -> Self {
        Self
    }
}

impl Authenticate for DeviceCodeFlow {
    type Error = Error;

    fn authenticate(
        &self,
        credentials: &mut Option<Credentials>,
        http_client: &HttpClient,
        server: &ServAddr,
    ) -> Result<(), Self::Error> {
        let client_id = http_client
            .get(format!("https://{server}/github"))
            .send()?
            .error_for_status()?
            .json::<AppInfo>()?
            .client_id;

        let code_response: CodeResponse = http_client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .form(&[("client_id", client_id.as_str())])
            .send()?
            .error_for_status()?
            .json()?;

        println!(
            "Enter code `{}` at `{}`.",
            code_response.user_code, code_response.verification_uri
        );

        let start = Instant::now();
        let expires_in = Duration::from_secs(code_response.expires_in);
        let mut interval = Duration::from_secs(code_response.interval);

        loop {
            if start.elapsed() >= expires_in {
                return Err(Error::Timeout);
            }

            let token_response: TokenResponse = http_client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&[
                    ("client_id", client_id.as_str()),
                    ("device_code", &code_response.device_code),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()?
                .error_for_status()?
                .json()?;

            match token_response {
                TokenResponse::Error(error) => match error.error.as_str() {
                    "authorization_pending" => {
                        sleep(interval);
                    }
                    "slow_down" => {
                        interval += Duration::from_secs(5);

                        sleep(interval);
                    }
                    "expired_token" => return Err(Error::Timeout),
                    s => return Err(Error::Other(s.to_string())),
                },
                TokenResponse::Success(success) => {
                    let now = OffsetDateTime::now_utc();
                    let mut token = success.access_token;

                    if let Some(expires_in) = success.expires_in {
                        token.set_expiry(now + time::Duration::seconds(expires_in));
                    }

                    let user = http_client
                        .get("https://api.github.com/user")
                        .header("Accept", "application/vnd.github+json")
                        .header("X-GitHub-Api-Version", "2026-03-10")
                        .bearer_auth(token.value())
                        .send()?
                        .error_for_status()?
                        .json::<UserResponse>()?
                        .login;

                    *credentials = Some(Credentials::new(user, token));

                    break;
                }
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
struct AppInfo {
    client_id: String,
}

#[derive(Deserialize)]
struct CodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: Url,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TokenResponse {
    Error(ErrorTokenResponse),
    Success(SuccessTokenResponse),
}

#[derive(Deserialize)]
struct ErrorTokenResponse {
    error: String,
}

#[serde_as]
#[derive(Deserialize)]
struct SuccessTokenResponse {
    #[serde_as(as = "DisplayFromStr")]
    access_token: Token,
    expires_in: Option<i64>,
}

#[derive(Deserialize)]
struct UserResponse {
    login: String,
}
