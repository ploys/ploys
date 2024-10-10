use jsonwebtoken::{Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::state::AppState;

use super::error::Error;

#[derive(Debug, Serialize)]
pub struct Claims {
    #[serde(with = "time::serde::timestamp")]
    iat: OffsetDateTime,
    #[serde(with = "time::serde::timestamp")]
    exp: OffsetDateTime,
    iss: String,
}

#[derive(Debug, Serialize)]
pub struct AccessTokenBody {
    pub repository_ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub token: String,
}

/// Gets an installation access token.
pub async fn get_installation_access_token(
    installation_id: u64,
    repository_id: u64,
    state: &AppState,
) -> Result<String, Error> {
    let claims = Claims {
        iat: OffsetDateTime::now_utc() - Duration::seconds(60),
        exp: OffsetDateTime::now_utc() + Duration::seconds(120),
        iss: state.github_app_client_id.as_ref().to_owned(),
    };

    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(state.github_app_private_key.as_bytes())?;
    let token = jsonwebtoken::encode(&header, &claims, &key)?;

    let client = Client::new();
    let response = client
        .post(format!(
            "https://api.github.com/app/installations/{installation_id}/access_tokens"
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "ploys")
        .json(&AccessTokenBody {
            repository_ids: vec![repository_id],
        })
        .send()
        .await?;

    let token_response = response.json::<AccessTokenResponse>().await?;

    Ok(token_response.token)
}
