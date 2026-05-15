use std::sync::{Arc, RwLock};

use keyring_core::CredentialStore;
use reqwest::blocking::Client as HttpClient;

use super::flows::Authenticate;
use super::flows::access_token::AccessTokenFlow;
use super::flows::device_code::DeviceCodeFlow;
use super::flows::keyring::KeyringFlow;
use super::flows::refresh_token::RefreshTokenFlow;
use super::{Client, Credentials, Error, ServAddr, Token};

/// The project management client builder.
#[derive(Clone, Debug, Default)]
pub struct Builder<T = ()> {
    server: ServAddr,
    auth_flow: T,
    credentials: Option<Credentials>,
}

impl Builder {
    /// Constructs a new project management client builder.
    pub fn new() -> Self {
        Self {
            server: ServAddr::default(),
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
    pub fn with_device_code_flow(self) -> Builder<DeviceCodeFlow> {
        self.with_authentication_flow(DeviceCodeFlow::new())
    }
}

impl<T> Builder<T> {
    /// Builds the client with the refresh token flow.
    ///
    /// See [`RefreshTokenFlow`] for more information about this authentication
    /// flow adapter.
    pub fn with_refresh_token_flow(self) -> Builder<RefreshTokenFlow<T>> {
        self.map_authentication_flow(RefreshTokenFlow::new)
    }

    /// Builds the client with the given keyring credential store.
    ///
    /// See [`KeyringFlow`] for more information about this authentication
    /// flow adapter.
    pub fn with_keyring_store(self, store: Arc<CredentialStore>) -> Builder<KeyringFlow<T>> {
        self.map_authentication_flow(|auth_flow| KeyringFlow::new(auth_flow).with_store(store))
    }

    /// Builds the client with the default keyring credential store.
    ///
    /// See [`KeyringFlow`] for more information about this authentication
    /// flow adapter.
    pub fn with_keyring_store_default(self) -> Builder<KeyringFlow<T>> {
        self.map_authentication_flow(|auth_flow| KeyringFlow::new(auth_flow))
    }
}

impl<T> Builder<T> {
    /// Sets the server address.
    pub fn set_server(&mut self, server: impl Into<ServAddr>) {
        self.server = server.into();
    }

    /// Builds the client with the given server address.
    pub fn with_server(mut self, server: impl Into<ServAddr>) -> Self {
        self.set_server(server);
        self
    }

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
            server: self.server,
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
            server: self.server,
            auth_flow: Arc::new(self.auth_flow),
            credentials: Arc::new(RwLock::new(self.credentials)),
            http_client: HttpClient::builder()
                .user_agent(concat!("ploys/", env!("CARGO_PKG_VERSION")))
                .build()?,
        })
    }
}
