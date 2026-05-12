mod builder;
mod credentials;
mod error;
pub mod flows;
mod server;

use std::sync::{Arc, RwLock};

use reqwest::blocking::Client as HttpClient;

use crate::project::{Error as ProjError, Project};
use crate::repository::RepoAddr;
use crate::repository::types::github::{Error as RepoError, GitHub};

pub use self::builder::Builder;
pub use self::credentials::{Credentials, Token, TokenError, TokenType};
pub use self::error::Error;
pub use self::server::ServAddr;

use self::flows::DynAuthenticate;

/// The project management client.
#[derive(Clone, Debug)]
pub struct Client {
    server: ServAddr,
    auth_flow: Arc<dyn DynAuthenticate>,
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
    pub(crate) fn authenticate(
        &self,
    ) -> Result<Option<Token>, Box<dyn std::error::Error + Send + Sync>> {
        let mut credentials = self
            .credentials
            .write()
            .unwrap_or_else(|err| err.into_inner());

        if credentials.is_none() || credentials.as_ref().is_some_and(|c| c.is_expired()) {
            self.auth_flow
                .dyn_authenticate(&mut credentials, &self.http_client, &self.server)?;
        }

        match &*credentials {
            Some(credentials) => Ok(Some(credentials.access_token().clone())),
            None => Ok(None),
        }
    }
}
