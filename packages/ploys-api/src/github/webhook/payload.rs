use std::borrow::Cow;

use axum::body::Bytes;
use axum::extract::rejection::{BytesRejection, ExtensionRejection};
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, RequestExt};
use axum_extra::TypedHeader;
use axum_extra::headers::ContentType;
use axum_extra::typed_header::TypedHeaderRejection;
use hmac::{Hmac, Mac};
use mime::Mime;
use serde::Deserialize;
use serde_json::Value;
use serde_json::error::Category;
use sha2::Sha256;

use super::header::{XGitHubEvent, XHubSignature256};
use super::secret::WebhookSecret;

/// The GitHub event payload.
#[derive(Debug)]
pub enum Payload {
    PullRequest(PullRequestPayload),
    RepositoryDispatch(RepositoryDispatchPayload),
    #[allow(dead_code)]
    Other(String, Value),
}

impl Payload {
    /// Gets the event name.
    pub fn event_name(&self) -> &str {
        match self {
            Payload::PullRequest(_) => "pull_request",
            Payload::RepositoryDispatch(_) => "repository_dispatch",
            Payload::Other(name, _) => name,
        }
    }
}

impl<S> FromRequest<S> for Payload
where
    S: Send + Sync,
{
    type Rejection = PayloadRejection;

    async fn from_request(mut req: Request, _: &S) -> Result<Self, Self::Rejection> {
        let content_type = req.extract_parts::<TypedHeader<ContentType>>().await?;
        let mime: Mime = content_type.0.into();
        let is_json = mime.type_() == "application"
            && (mime.subtype() == "json" || mime.suffix().is_some_and(|name| name == "json"));

        if !is_json {
            return Err(PayloadRejection::ContentType);
        }

        let event = req.extract_parts::<TypedHeader<XGitHubEvent>>().await?;
        let signature = req.extract_parts::<TypedHeader<XHubSignature256>>().await?;
        let secret = req.extract_parts::<Extension<WebhookSecret>>().await?;
        let bytes = req.extract::<Bytes, _>().await?;

        let mut hmac = Hmac::<Sha256>::new_from_slice(secret.value.as_bytes())
            .expect("HMAC can take key of any size");

        hmac.update(&bytes);

        let digest = hmac.finalize().into_bytes();
        let hex = hex::encode(digest);

        if hex != signature.value() {
            return Err(PayloadRejection::Signature);
        }

        let event = event.0.into_inner();

        Ok(match event.as_str() {
            "pull_request" => Self::PullRequest(serde_json::from_slice(&bytes)?),
            "repository_dispatch" => Self::RepositoryDispatch(serde_json::from_slice(&bytes)?),
            _ => Self::Other(event, serde_json::from_slice(&bytes)?),
        })
    }
}

/// The `pull_request` webhook payload.
#[derive(Debug, Deserialize)]
pub struct PullRequestPayload {
    pub action: String,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub installation: Installation,
}

/// The `repository_dispatch` webhook payload.
#[derive(Debug, Deserialize)]
pub struct RepositoryDispatchPayload {
    pub action: String,
    pub branch: String,
    pub client_payload: Value,
    pub repository: Repository,
    pub installation: Installation,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Installation {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub head: Branch,
    pub merged: bool,
    pub merge_commit_sha: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Branch {
    pub r#ref: String,
}

pub enum PayloadRejection {
    ContentType,
    Signature,
    Header(TypedHeaderRejection),
    Bytes(BytesRejection),
    Extension(ExtensionRejection),
    Json(serde_json::Error),
}

impl PayloadRejection {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::ContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::Signature => StatusCode::FORBIDDEN,
            Self::Header(_) => StatusCode::BAD_REQUEST,
            Self::Bytes(rej) => rej.status(),
            Self::Extension(rej) => rej.status(),
            Self::Json(err) => match err.classify() {
                Category::Data => StatusCode::UNPROCESSABLE_ENTITY,
                _ => StatusCode::BAD_REQUEST,
            },
        }
    }

    pub fn message(&self) -> Cow<'static, str> {
        match self {
            Self::ContentType => {
                Cow::Borrowed("Expected request with `Content-Type: application/json`")
            }
            Self::Signature => Cow::Borrowed("Invalid SHA256 signature."),
            Self::Header(rej) => Cow::Owned(rej.to_string()),
            Self::Bytes(rej) => Cow::Owned(rej.body_text()),
            Self::Extension(rej) => Cow::Owned(rej.body_text()),
            Self::Json(err) => Cow::Owned(err.to_string()),
        }
    }
}

impl IntoResponse for PayloadRejection {
    fn into_response(self) -> Response {
        (self.status(), self.message()).into_response()
    }
}

impl From<TypedHeaderRejection> for PayloadRejection {
    fn from(value: TypedHeaderRejection) -> Self {
        Self::Header(value)
    }
}

impl From<BytesRejection> for PayloadRejection {
    fn from(value: BytesRejection) -> Self {
        Self::Bytes(value)
    }
}

impl From<ExtensionRejection> for PayloadRejection {
    fn from(value: ExtensionRejection) -> Self {
        Self::Extension(value)
    }
}

impl From<serde_json::Error> for PayloadRejection {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
