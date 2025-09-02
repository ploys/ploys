use std::borrow::Cow;
use std::fmt::{self, Display};
use std::string::FromUtf8Error;

use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

/// The webhook response error.
#[derive(Debug)]
pub enum Error {
    Jwt(jsonwebtoken::errors::Error),
    Request(reqwest::Error),
    Project(ploys::project::Error<ploys::repository::types::github::Error>),
    Utf8(FromUtf8Error),
    Json(serde_json::Error),
}

impl Error {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Jwt(_) => StatusCode::FORBIDDEN,
            Self::Request(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Project(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Utf8(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Json(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    pub fn message(&self) -> Cow<'static, str> {
        match self {
            Self::Jwt(error) => Cow::Owned(format!("JWT: {error}")),
            Self::Request(error) => Cow::Owned(format!("Request: {error}")),
            Self::Project(error) => Cow::Owned(format!("Project: {error}")),
            Self::Utf8(error) => Cow::Owned(format!("UTF-8: {error}")),
            Self::Json(error) => Cow::Owned(format!("JSON: {error}")),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.status(), self.message()).into_response()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Jwt(err) => Some(err),
            Self::Request(err) => Some(err),
            Self::Project(err) => Some(err),
            Self::Utf8(err) => Some(err),
            Self::Json(err) => Some(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        Self::Jwt(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

impl From<ploys::project::Error<ploys::repository::types::github::Error>> for Error {
    fn from(error: ploys::project::Error<ploys::repository::types::github::Error>) -> Self {
        Self::Project(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Self::Utf8(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
