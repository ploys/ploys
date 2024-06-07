use std::borrow::Cow;

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Json, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, RequestExt};
use axum_extra::typed_header::TypedHeaderRejection;
use axum_extra::TypedHeader;
use serde_json::Value;

use super::header::XGitHubEvent;

/// The GitHub event payload.
pub struct Payload {
    /// The event name.
    pub event: String,
    /// The payload value.
    pub value: Value,
}

#[async_trait]
impl<S> FromRequest<S> for Payload
where
    S: Send + Sync,
{
    type Rejection = PayloadRejection;

    async fn from_request(mut req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let event = req.extract_parts::<TypedHeader<XGitHubEvent>>().await?;
        let value = Json::from_request(req, state).await?;

        Ok(Self {
            event: event.0.into_inner(),
            value: value.0,
        })
    }
}

pub enum PayloadRejection {
    Header(TypedHeaderRejection),
    Json(JsonRejection),
}

impl PayloadRejection {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Header(_) => StatusCode::BAD_REQUEST,
            Self::Json(rej) => rej.status(),
        }
    }

    pub fn message(&self) -> Cow<'static, str> {
        match self {
            Self::Header(rej) => Cow::Owned(rej.to_string()),
            Self::Json(rej) => Cow::Owned(rej.body_text()),
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

impl From<JsonRejection> for PayloadRejection {
    fn from(rejection: JsonRejection) -> Self {
        Self::Json(rejection)
    }
}
