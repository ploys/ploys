use std::sync::{Arc, RwLock};

use reqwest::blocking::Client as HttpClient;

use super::flows::Authenticate;
use super::flows::access_token::AccessTokenFlow;
use super::flows::device_code::DeviceCodeFlow;
use super::{Client, Credentials, Error, ServAddr, Token};

/// The project management client builder.
#[derive(Clone, Debug, Default)]
pub struct Builder<T = ()> {
    auth_flow: T,
    credentials: Option<Credentials>,
}

impl Builder {
    /// Constructs a new project management client builder.
    pub fn new() -> Self {
        Self {
            auth_flow: (),
            credentials: None,
        }
    }

    /// Builds the client with the given authentication flow.
    pub fn with_authentication_flow<T>(self, auth_flow: T) -> Builder<T>
    where
        T: Authenticate,
    {
        self.map_authentication_flow(|_| auth_flow)
    }

    /// Builds the client with the access token authentication flow.
    ///
    /// See [`AccessTokenFlow`] for more information about this authentication
    /// flow.
    pub fn with_access_token_flow(self, token: impl Into<Token>) -> Builder<AccessTokenFlow> {
        self.with_authentication_flow(AccessTokenFlow::new(token))
    }

    /// Builds the client with the device code authentication flow.
    ///
    /// See [`DeviceCodeFlow`] for more information about this authentication
    /// flow.
    pub fn with_device_code_flow(self, server: impl Into<ServAddr>) -> Builder<DeviceCodeFlow> {
        self.with_authentication_flow(DeviceCodeFlow::new(server))
    }
}

impl<T> Builder<T> {
    /// Sets the client authentication credentials.
    pub fn set_credentials(&mut self, credentials: impl Into<Credentials>) {
        self.credentials = Some(credentials.into());
    }

    /// Builds the client with the given authentication credentials.
    pub fn with_credentials(mut self, credentials: impl Into<Credentials>) -> Self {
        self.set_credentials(credentials);
        self
    }

    /// Maps the authentication flow.
    pub fn map_authentication_flow<U>(self, f: impl FnOnce(T) -> U) -> Builder<U> {
        Builder {
            auth_flow: f(self.auth_flow),
            credentials: self.credentials,
        }
    }
}

impl<T> Builder<T>
where
    T: Authenticate,
{
    /// Finishes building the client.
    pub fn finished(self) -> Result<Client, Error> {
        Ok(Client {
            auth_flow: Arc::new(self.auth_flow),
            credentials: Arc::new(RwLock::new(self.credentials)),
            http_client: HttpClient::builder()
                .user_agent(concat!("ploys/", env!("CARGO_PKG_VERSION")))
                .build()?,
        })
    }
}
