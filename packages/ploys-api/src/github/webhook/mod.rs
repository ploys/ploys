mod auth;
mod error;
mod header;
mod payload;
pub mod secret;

use axum::extract::State;
use ploys::changelog::Changelog;
use ploys::package::BumpOrVersion;
use ploys::project::repository::revision::Revision;
use ploys::project::Project;
use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::state::AppState;

use self::auth::get_installation_access_token;
use self::error::Error;
use self::payload::{Payload, RefType, RepositoryDispatchPayload};

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
        Payload::PullRequest(payload) => match &*payload.action {
            "closed" if payload.pull_request.merged => {
                if payload.pull_request.head.r#ref.starts_with("release/") {
                    if let Some(sha) = payload.pull_request.merge_commit_sha {
                        create_release(
                            &payload.pull_request.head.r#ref[8..],
                            &sha,
                            payload.installation.id,
                            payload.repository.id,
                            &payload.repository.full_name,
                            &state,
                        )
                        .await?;
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        },
        Payload::RepositoryDispatch(payload) => match &*payload.action {
            "ploys-package-release-initiate" => {
                initiate_release(payload, &state).await?;

                Ok(())
            }
            _ => Ok(()),
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
    let package = project.packages().find(|package| {
        release.starts_with(package.name())
            && release.as_bytes().get(package.name().len()) == Some(&b'-')
            && release[package.name().len() + 1..]
                .parse::<Version>()
                .is_ok()
    });

    let (package, path, version) = match package {
        Some(package) => (
            package.name().to_owned(),
            package.path(),
            release[package.name().len() + 1..]
                .parse::<Version>()
                .map_err(|_| Error::Payload)?,
        ),
        None => {
            let version = release.parse::<Version>().map_err(|_| Error::Payload)?;
            let package = project
                .packages()
                .find(|package| package.name() == project.name())
                .ok_or_else(|| ploys::project::Error::PackageNotFound(project.name().to_owned()))?;

            (package.name().to_owned(), package.path(), version)
        }
    };

    let client = Client::new();
    let message = match package == project.name() {
        true => format!("Release `{version}`"),
        false => format!("Release `{package}@{version}`"),
    };

    let changelog_path = path.parent().expect("parent").join("CHANGELOG.md");

    let mut files = Vec::new();
    let mut changelog = match project.get_file_contents(&changelog_path).ok() {
        Some(bytes) => String::from_utf8(bytes)?
            .parse::<Changelog>()
            .expect("changelog"),
        None => Changelog::new(),
    };

    let mut changelog_release = project.get_changelog_release(&package, version.to_string())?;

    changelog.add_release(changelog_release.clone());
    files.push((changelog_path, changelog.to_string()));
    project.set_package_version(&package, version.clone())?;
    project.commit(&message, files)?;
    changelog_release.set_description(format!(
        "Releasing package `{package}` version `{version}`."
    ));

    if let Some(url) = changelog_release.url() {
        changelog_release.add_reference(version.to_string(), url.to_string());
    }

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
            body: changelog_release.to_string(),
        })
        .send()
        .await?
        .json::<PullRequestResponse>()
        .await?
        .id;

    println!("Created pull request {issue_number}.");

    Ok(())
}

/// Creates a new release.
async fn create_release(
    release: &str,
    sha: &str,
    installation_id: u64,
    repository_id: u64,
    repository_name: &str,
    state: &AppState,
) -> Result<(), Error> {
    let token = get_installation_access_token(installation_id, repository_id, state).await?;
    let revision = Revision::sha(sha);
    let project =
        Project::github_with_revision_and_authentication_token(repository_name, revision, &token)?;
    let package = project.packages().find(|package| {
        release.starts_with(package.name())
            && release.as_bytes().get(package.name().len()) == Some(&b'-')
            && release[package.name().len() + 1..]
                .parse::<Version>()
                .is_ok()
    });

    let (package, path, version) = match package {
        Some(package) => (
            package.name().to_owned(),
            package.path(),
            release[package.name().len() + 1..]
                .parse::<Version>()
                .map_err(|_| Error::Payload)?,
        ),
        None => {
            let version = release.parse::<Version>().map_err(|_| Error::Payload)?;
            let package = project
                .packages()
                .find(|package| package.name() == project.name())
                .ok_or_else(|| ploys::project::Error::PackageNotFound(project.name().to_owned()))?;

            (package.name().to_owned(), package.path(), version)
        }
    };

    let name = match package == project.name() {
        true => format!("{version}"),
        false => format!("{package} {version}"),
    };

    let tag_name = match package == project.name() {
        true => format!("{version}"),
        false => format!("{package}-{version}"),
    };

    let changelog_path = path.parent().expect("parent").join("CHANGELOG.md");
    let changelog = match project.get_file_contents(changelog_path).ok() {
        Some(bytes) => String::from_utf8(bytes)?
            .parse::<Changelog>()
            .expect("changelog"),
        None => Changelog::new(),
    };

    let body = match changelog.get_release(version.to_string()) {
        Some(release) => format!("{release:#}"),
        None => {
            let release = project.get_changelog_release(&package, version.to_string())?;

            format!("{release:#}")
        }
    };

    let body = body.lines().skip(2).collect::<Vec<_>>().join("\n");
    let prerelease = !version.pre.is_empty();
    let is_latest = package == project.name() && !prerelease;

    let client = Client::new();
    let release_id = client
        .post(format!(
            "https://api.github.com/repos/{repository_name}/releases",
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "ploys/ploys")
        .json(&CreateRelease {
            tag_name,
            target_commitish: sha.to_owned(),
            name,
            body,
            draft: false,
            prerelease,
            generate_release_notes: false,
            make_latest: is_latest.into(),
        })
        .send()
        .await?
        .json::<ReleaseResponse>()
        .await?
        .id;

    println!("Created release {release_id}");

    Ok(())
}

/// Initiates the release process.
///
/// This does not yet support parallel release branches so simply ensures that
/// all new versions are greater than the previous.
async fn initiate_release(
    payload: RepositoryDispatchPayload,
    state: &AppState,
) -> Result<(), Error> {
    let ClientPayload { package, version } = serde_json::from_value(payload.client_payload)?;

    let token =
        get_installation_access_token(payload.installation.id, payload.repository.id, state)
            .await?;

    let mut project = Project::github_with_revision_and_authentication_token(
        &payload.repository.full_name,
        Revision::branch(&payload.branch),
        &token,
    )?;

    let version = match version {
        BumpOrVersion::Bump(bump) => {
            project.bump_package_version(&package, bump)?;
            project
                .packages()
                .find(|pkg| pkg.name() == package)
                .ok_or_else(|| ploys::project::Error::PackageNotFound(package.clone()))?
                .version()
                .parse::<Version>()
                .map_err(ploys::package::BumpError::Semver)
                .map_err(ploys::project::Error::Bump)?
        }
        BumpOrVersion::Version(version) => {
            let current_version = project
                .packages()
                .find(|pkg| pkg.name() == package)
                .ok_or_else(|| ploys::project::Error::PackageNotFound(package.clone()))?
                .version()
                .parse::<Version>()
                .map_err(ploys::package::BumpError::Semver)
                .map_err(ploys::project::Error::Bump)?;

            if version <= current_version {
                return Err(
                    ploys::project::Error::Bump(ploys::package::BumpError::Invalid(
                        version.to_string(),
                    ))
                    .into(),
                );
            }

            version
        }
    };

    let client = Client::new();

    let sha = client
        .get(format!(
            "https://api.github.com/repos/{}/git/ref/heads/{}",
            payload.repository.full_name, payload.branch,
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "ploys/ploys")
        .send()
        .await?
        .json::<RefResponse>()
        .await?
        .object
        .sha;

    client
        .post(format!(
            "https://api.github.com/repos/{}/git/refs",
            payload.repository.full_name,
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "ploys/ploys")
        .json(&NewBranch {
            r#ref: match package == project.name() {
                true => format!("refs/heads/release/{version}"),
                false => format!("refs/heads/release/{package}-{version}"),
            },
            sha,
        })
        .send()
        .await?;

    Ok(())
}

#[serde_as]
#[derive(Deserialize)]
struct ClientPayload {
    package: String,
    #[serde_as(as = "DisplayFromStr")]
    version: BumpOrVersion,
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

#[derive(Serialize)]
struct CreateRelease {
    tag_name: String,
    target_commitish: String,
    name: String,
    body: String,
    draft: bool,
    prerelease: bool,
    generate_release_notes: bool,
    make_latest: MakeLatest,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum MakeLatest {
    True,
    False,
}

impl From<bool> for MakeLatest {
    fn from(value: bool) -> Self {
        match value {
            true => Self::True,
            false => Self::False,
        }
    }
}

#[derive(Deserialize)]
struct ReleaseResponse {
    id: u64,
}

#[derive(Deserialize)]
struct RefResponse {
    object: Object,
}

#[derive(Deserialize)]
struct Object {
    sha: String,
}

#[derive(Serialize)]
struct NewBranch {
    r#ref: String,
    sha: String,
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
