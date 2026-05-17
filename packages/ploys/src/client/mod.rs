mod builder;
mod credentials;
mod error;
pub mod flows;
mod projects;
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

use self::error::MissingCredentials;
use self::flows::DynAuthenticate;
use self::projects::Projects;

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

    /// Iterates over managed projects.
    ///
    /// Note that this method currently only supports credentials generated via
    /// the device code flow.
    pub fn projects(&self) -> impl Iterator<Item = Result<Project<GitHub>, Error>> {
        Projects::new(self)
    }
}

impl Client {
    /// Obtains the authentication credentials.
    ///
    /// This method returns cached credentials if they have not yet expired, or
    /// initiates a new authentication flow to request updated credentials if
    /// they have expired. The returned credentials may no longer be valid if
    /// they have been externally revoked or the expiration date is unknown.
    pub fn login(&self) -> Result<Credentials, Error> {
        match self.get_credentials().map_err(Error::Auth)? {
            Some(credentials) => Ok(credentials),
            None => Err(Error::Auth(Box::new(MissingCredentials))),
        }
    }
}

impl Client {
    /// Gets the HTTP client.
    pub(crate) fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Gets the authentication credentials.
    pub(crate) fn get_credentials(
        &self,
    ) -> Result<Option<Credentials>, Box<dyn std::error::Error + Send + Sync>> {
        let mut credentials = self
            .credentials
            .write()
            .unwrap_or_else(|err| err.into_inner());

        if credentials.is_none()
            || credentials
                .as_ref()
                .is_some_and(|credentials| credentials.access_token().is_expired())
        {
            self.auth_flow
                .dyn_authenticate(&mut credentials, &self.http_client, &self.server)?;
        }

        Ok(credentials.clone())
    }

    /// Gets the authentication credentials access token.
    pub(crate) fn authenticate(
        &self,
    ) -> Result<Option<Token>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_credentials()?
            .map(|credentials| credentials.access_token().clone()))
    }
}
