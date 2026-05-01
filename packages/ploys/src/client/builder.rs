use std::sync::{Arc, RwLock};

use reqwest::blocking::Client as HttpClient;

use super::{Client, Credentials, Error};

/// The project management client builder.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    credentials: Option<Credentials>,
}

impl Builder {
    /// Constructs a new project management client builder.
    pub fn new() -> Self {
        Self { credentials: None }
    }

    /// Builds the client with the given authentication credentials.
    pub fn with_credentials(mut self, credentials: impl Into<Credentials>) -> Self {
        self.set_credentials(credentials);
        self
    }

    /// Finishes building the client.
    pub fn finished(self) -> Result<Client, Error> {
        Ok(Client {
            credentials: Arc::new(RwLock::new(self.credentials)),
            http_client: HttpClient::builder()
                .user_agent(concat!("ploys/", env!("CARGO_PKG_VERSION")))
                .build()?,
        })
    }
}

impl Builder {
    /// Sets the client authentication credentials.
    pub fn set_credentials(&mut self, credentials: impl Into<Credentials>) {
        self.credentials = Some(credentials.into());
    }
}
