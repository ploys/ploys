mod auth;
mod error;
mod header;
mod payload;
pub mod secret;

use axum::extract::State;
use axum_extra::TypedHeader;
use ploys::package::BumpOrVersion;
use ploys::project::Project;
use ploys::repository::revision::Revision;
use semver::Version;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, error, instrument};

use crate::state::AppState;

use self::auth::get_installation_access_token;
use self::error::Error;
use self::header::XGitHubDelivery;
use self::payload::{Payload, PullRequestPayload, RepositoryDispatchPayload};

/// Receives the GitHub webhook event payload.
#[instrument(skip_all, fields(delivery = %delivery.into_inner(), event_name = payload.event_name()))]
pub async fn receive(
    state: State<AppState>,
    delivery: TypedHeader<XGitHubDelivery>,
    payload: Payload,
) -> Result<(), Error> {
    debug!(?payload, "Received webhook event");

    match payload {
        Payload::PullRequest(payload) => match &*payload.action {
            "closed" if payload.pull_request.merged => {
                if payload.pull_request.head.r#ref.starts_with("release/")
                    && let Some(sha) = &payload.pull_request.merge_commit_sha
                {
                    create_release(
                        payload.pull_request.head.r#ref[8..].to_owned(),
                        sha.clone(),
                        payload,
                        &state,
                    )
                    .await?;
                }

                Ok(())
            }
            _ => Ok(()),
        },
        Payload::RepositoryDispatch(payload) => match &*payload.action {
            "ploys-package-release-request" => {
                request_release(payload, &state).await?;

                Ok(())
            }
            _ => Ok(()),
        },
        Payload::Other(_, _) => Ok(()),
    }
}

/// Creates a new release.
async fn create_release(
    release: String,
    sha: String,
    payload: PullRequestPayload,
    state: &AppState,
) -> Result<(), Error> {
    let token =
        get_installation_access_token(payload.installation.id, payload.repository.id, state)
            .await?;

    tokio::task::spawn_blocking(move || {
        if let Err(err) = create_release_sync(token, release, sha, payload) {
            error!("Error creating release: {err}");
        }
    });

    Ok(())
}

/// Creates a new release.
fn create_release_sync(
    token: String,
    release: String,
    sha: String,
    payload: PullRequestPayload,
) -> Result<(), Error> {
    let project = Project::github_with_revision_and_authentication_token(
        &payload.repository.full_name,
        Revision::sha(sha),
        &token,
    )?;

    let package = project
        .packages()
        .find(|package| match package.is_primary() {
            true => release.parse::<Version>().ok() == Some(package.version()),
            false => {
                release.starts_with(package.name())
                    && release.as_bytes().get(package.name().len()) == Some(&b'-')
                    && release[package.name().len() + 1..].parse::<Version>().ok()
                        == Some(package.version())
            }
        })
        .ok_or(ploys::package::Error::NotFound(release))
        .map_err(ploys::project::Error::Package)?;

    project
        .create_package_release(package.name())?
        .finish()
        .map_err(ploys::project::Error::Repository)?;

    Ok(())
}

/// Requests the package release.
///
/// This does not yet support parallel release branches so simply ensures that
/// all new versions are greater than the previous.
async fn request_release(
    payload: RepositoryDispatchPayload,
    state: &AppState,
) -> Result<(), Error> {
    let token =
        get_installation_access_token(payload.installation.id, payload.repository.id, state)
            .await?;

    tokio::task::spawn_blocking(move || {
        if let Err(err) = create_release_request(token, payload) {
            error!("Error creating release request: {err}");
        }
    });

    Ok(())
}

/// Creates the release request.
///
/// The `Project` is currently using blocking requests so this should be spawned
/// using `tokio::task::spawn_blocking`. This also avoids any timeout issues for
/// completing the webhook event request.
fn create_release_request(token: String, payload: RepositoryDispatchPayload) -> Result<(), Error> {
    let ClientPayload { package, version } = serde_json::from_value(payload.client_payload)?;

    let project = Project::github_with_revision_and_authentication_token(
        &payload.repository.full_name,
        Revision::branch(&payload.branch),
        &token,
    )?;

    let package = project
        .get_package(&package)
        .ok_or(ploys::package::Error::NotFound(package))
        .map_err(ploys::project::Error::Package)?;

    project
        .create_package_release_request(package.name(), version)?
        .finish()?;

    Ok(())
}

#[serde_as]
#[derive(Deserialize)]
struct ClientPayload {
    package: String,
    #[serde_as(as = "DisplayFromStr")]
    version: BumpOrVersion,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use axum::routing::post;
    use axum::{Extension, Router};
    use hmac::{Hmac, Mac};
    use serde_json::{Value, json};
    use sha2::Sha256;
    use tower_service::Service;

    use crate::state::AppState;

    use super::secret::WebhookSecret;

    fn router() -> Router {
        Router::new().route(
            "/github/webhook",
            post(super::receive)
                .layer(Extension(WebhookSecret {
                    value: String::from("super_secret"),
                }))
                .with_state(AppState {
                    github_app_client_id: Arc::from(""),
                    github_app_private_key: Arc::from(""),
                }),
        )
    }

    fn payload() -> Value {
        json!({
            "action": "opened",
            "issue": {
              "url": "https://api.github.com/repos/octocat/Hello-World/issues/1347",
              "number": 1347
            },
            "repository" : {
              "id": 1296269,
              "full_name": "octocat/Hello-World",
              "owner": {
                  "login": "octocat",
                  "id": 1
              }
            },
            "sender": {
              "login": "octocat",
              "id": 1
            }
        })
    }

    #[tokio::test]
    async fn test_webhook_endpoint_valid_signature() {
        let mut router = router();
        let payload = serde_json::to_string(&payload()).unwrap();
        let mut hmac = Hmac::<Sha256>::new_from_slice(b"super_secret").unwrap();

        hmac.update(payload.as_bytes());

        let digest = hmac.finalize().into_bytes();
        let hex = hex::encode(digest);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/github/webhook")
            .header("Content-Type", "application/json")
            .header("X-GitHub-Event", "issues")
            .header("X-GitHub-Delivery", "00000000-0000-0000-0000-000000000000")
            .header("X-Hub-Signature-256", format!("sha256={hex}"))
            .body(Body::from(payload))
            .unwrap();

        let response = router.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_webhook_endpoint_invalid_signature() {
        let mut router = router();
        let payload = serde_json::to_string(&payload()).unwrap();
        let mut hmac = Hmac::<Sha256>::new_from_slice(b"not_super_secret").unwrap();

        hmac.update(payload.as_bytes());

        let digest = hmac.finalize().into_bytes();
        let hex = hex::encode(digest);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/github/webhook")
            .header("Content-Type", "application/json")
            .header("X-GitHub-Event", "issues")
            .header("X-GitHub-Delivery", "00000000-0000-0000-0000-000000000000")
            .header("X-Hub-Signature-256", format!("sha256={hex}"))
            .body(Body::from(payload))
            .unwrap();

        let response = router.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
