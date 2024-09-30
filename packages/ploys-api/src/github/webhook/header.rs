use std::fmt::{self, Display};

use axum::http::{HeaderName, HeaderValue};
use axum_extra::headers::{Error, Header};

static X_GITHUB_EVENT: HeaderName = HeaderName::from_static("x-github-event");
static X_HUB_SIGNATURE_256: HeaderName = HeaderName::from_static("x-hub-signature-256");

pub struct XGitHubEvent {
    value: String,
}

impl XGitHubEvent {
    pub fn into_inner(self) -> String {
        self.value
    }
}

impl Display for XGitHubEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl Header for XGitHubEvent {
    fn name() -> &'static HeaderName {
        &X_GITHUB_EVENT
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
        Self: Sized,
    {
        Ok(Self {
            value: values
                .next()
                .ok_or_else(Error::invalid)?
                .to_str()
                .map_err(|_| Error::invalid())?
                .to_owned(),
        })
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        values.extend(std::iter::once(
            HeaderValue::from_str(&self.value).expect("valid header"),
        ));
    }
}

pub struct XHubSignature256 {
    value: String,
}

impl XHubSignature256 {
    pub fn value(&self) -> &str {
        &self.value["sha256=".len()..]
    }
}

impl Display for XHubSignature256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl Header for XHubSignature256 {
    fn name() -> &'static HeaderName {
        &X_HUB_SIGNATURE_256
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
        Self: Sized,
    {
        Ok(Self {
            value: values
                .find(|value| value.as_bytes().starts_with(b"sha256="))
                .ok_or_else(Error::invalid)?
                .to_str()
                .map_err(|_| Error::invalid())?
                .to_owned(),
        })
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        values.extend(std::iter::once(
            HeaderValue::from_str(&self.value).expect("valid header"),
        ));
    }
}
