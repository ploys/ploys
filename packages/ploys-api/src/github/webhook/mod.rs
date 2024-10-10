mod auth;
mod error;
mod header;
mod payload;
pub mod secret;

use axum::extract::State;
use ploys::project::source::revision::Revision;
use ploys::project::Project;
use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

use self::auth::get_installation_access_token;
use self::error::Error;
use self::payload::{Payload, RefType};

/// Receives the GitHub webhook event payload.
pub async fn receive(state: State<AppState>, payload: Payload) -> Result<(), Error> {
    match payload {
        Payload::Create(payload) => match payload.ref_type {
            RefType::Branch => {
                if payload.r#ref.starts_with("release/") {
                    create_release_pull_request(
                        &payload.r#ref[8..],
                        &payload.master_branch,
                        payload.installation.id,
                        payload.repository.id,
                        &payload.repository.full_name,
                        &state,
                    )
                    .await?;
                }

                Ok(())
            }
            RefType::Tag => Ok(()),
        },
        Payload::Other(event, payload) => {
            println!("Event: {event}");
            println!("Payload: {payload:#}");

            Ok(())
        }
    }
}

/// Creates a Pull Request for the given release.
pub async fn create_release_pull_request(
    release: &str,
    target_branch: &str,
    installation_id: u64,
    repository_id: u64,
    repository_name: &str,
    state: &AppState,
) -> Result<(), Error> {
    let token = get_installation_access_token(installation_id, repository_id, state).await?;
    let revision = Revision::branch(format!("release/{release}"));
    let mut project =
        Project::github_with_revision_and_authentication_token(repository_name, revision, &token)?;
    let package = project.packages().iter().find(|package| {
        release.starts_with(package.name())
            && release.as_bytes().get(package.name().len()) == Some(&b'-')
            && release[package.name().len() + 1..]
                .parse::<Version>()
                .is_ok()
    });

    let (package, version) = match package {
        Some(package) => (
            package.name().to_owned(),
            release[package.name().len() + 1..]
                .parse::<Version>()
                .map_err(|_| Error::Payload)?,
        ),
        None => {
            let version = release.parse::<Version>().map_err(|_| Error::Payload)?;
            let package = project
                .packages()
                .iter()
                .find(|package| package.name() == project.name())
                .ok_or_else(|| ploys::project::Error::PackageNotFound(project.name().to_owned()))?;

            (package.name().to_owned(), version)
        }
    };

    let client = Client::new();
    let message = match package == project.name() {
        true => format!("Release `{version}`"),
        false => format!("Release `{package}@{version}`"),
    };

    project.set_package_version(&package, version.clone())?;
    project.commit(&message)?;

    let issue_number = client
        .post(format!(
            "https://api.github.com/repos/{repository_name}/pulls"
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "ploys/ploys")
        .json(&CreatePullRequest {
            title: message,
            head: format!("release/{release}"),
            base: target_branch.to_owned(),
            body: format!("Releasing package `{package}` version `{version}`."),
        })
        .send()
        .await?
        .json::<PullRequestResponse>()
        .await?
        .id;

    println!("Created pull request {issue_number}.");

    Ok(())
}

#[derive(Serialize)]
struct CreatePullRequest {
    title: String,
    head: String,
    base: String,
    body: String,
}

#[derive(Deserialize)]
struct PullRequestResponse {
    id: u64,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use axum::routing::post;
    use axum::{Extension, Router};
    use hmac::{Hmac, Mac};
    use serde_json::{json, Value};
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
            .header("X-Hub-Signature-256", format!("sha256={hex}"))
            .body(Body::from(payload))
            .unwrap();

        let response = router.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
