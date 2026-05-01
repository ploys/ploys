mod builder;
mod credentials;
mod error;

use std::sync::{Arc, RwLock};

use reqwest::blocking::Client as HttpClient;

use crate::project::{Error as ProjError, Project};
use crate::repository::RepoAddr;
use crate::repository::types::github::{Error as RepoError, GitHub};

pub use self::builder::Builder;
pub use self::credentials::{Credentials, Token, TokenError};
pub use self::error::Error;

/// The project management client.
#[derive(Clone, Debug)]
pub struct Client {
    credentials: Arc<RwLock<Option<Credentials>>>,
    http_client: HttpClient,
}

impl Client {
    /// Constructs a new project management client.
    pub fn new(credentials: impl Into<Credentials>) -> Result<Self, Error> {
        Self::build().with_credentials(credentials).finished()
    }

    /// Build a new project management client.
    pub fn build() -> Builder {
        Builder::new()
    }
}

impl Client {
    /// Gets a project with the given repository address.
    pub fn get_project<R>(&self, repo: R) -> Result<Project<GitHub>, Error>
    where
        R: TryInto<RepoAddr, Error: Into<RepoError>>,
    {
        let repo = GitHub::new(self.clone(), repo).map_err(ProjError::Repository)?;
        let proj = Project::open(repo)?;

        Ok(proj)
    }
}

impl Client {
    /// Gets the HTTP client.
    pub(crate) fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Gets the authentication credentials access token.
    pub(crate) fn get_access_token(&self) -> Option<Token> {
        let credentials = self
            .credentials
            .read()
            .unwrap_or_else(|err| err.into_inner());

        match &*credentials {
            Some(credentials) => credentials.get_access_token(),
            None => None,
        }
    }
}
