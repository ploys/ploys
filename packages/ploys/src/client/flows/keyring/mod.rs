mod error;

use std::collections::HashMap;
use std::io::IsTerminal;
use std::sync::Arc;

use dialoguer::Select;
use keyring_core::CredentialStore;
use reqwest::blocking::Client as HttpClient;

use crate::client::{Credentials, ServAddr};

pub use self::error::Error;

use super::Authenticate;

/// The keyring authentication flow adapter.
///
/// Note that this defaults to using the default shared credential store if it
/// exists. Otherwise, this can be overridden by calling [`Self::with_store`]
/// or [`Self::set_store`] when building the authentication flow adapter.
#[derive(Clone, Debug, Default)]
pub struct KeyringFlow<T> {
    store: Option<Arc<CredentialStore>>,
    auth_flow: T,
}

impl<T> KeyringFlow<T> {
    /// Constructs a new keyring authentication flow adapter.
    pub fn new(auth_flow: T) -> Self {
        Self {
            store: None,
            auth_flow,
        }
    }
}

impl<T> KeyringFlow<T> {
    /// Sets the credential store.
    pub fn set_store(&mut self, store: Arc<CredentialStore>) {
        self.store = Some(store);
    }

    /// Builds the flow with the given credential store.
    pub fn with_store(mut self, store: Arc<CredentialStore>) -> Self {
        self.set_store(store);
        self
    }
}

impl<T> KeyringFlow<T>
where
    T: Authenticate,
{
    /// Gets the configured store or the default store.
    fn get_store(&self) -> Result<Arc<CredentialStore>, Error<T::Error>> {
        self.store
            .as_ref()
            .cloned()
            .or_else(keyring_core::get_default_store)
            .ok_or_else(|| keyring_core::Error::NoDefaultStore)
            .map_err(Error::Keyring)
    }
}

impl<T> Authenticate for KeyringFlow<T>
where
    T: Authenticate,
{
    type Error = Error<T::Error>;

    fn authenticate(
        &self,
        credentials: &mut Option<Credentials>,
        http_client: &HttpClient,
        server: &ServAddr,
    ) -> Result<(), Self::Error> {
        let store = self.get_store()?;
        let service = format!("ploys:{server}");
        let entry = match credentials {
            Some(credentials) => Some(store.build(&service, credentials.user(), None)?),
            None => {
                let mut options = store.search(&{
                    let mut map = HashMap::new();

                    map.insert("service", &*service);
                    map
                })?;

                if options.len() > 1 && std::io::stdin().is_terminal() {
                    let mut select = Select::new().with_prompt("User");

                    for option in &options {
                        if let Some((svc, user)) = option.get_specifiers() {
                            select = select.item(format!("{user} ({svc})"));
                        } else {
                            select = select.item(String::from("unknown (unknown)"));
                        }
                    }

                    match select.interact_opt()? {
                        Some(option) => options.into_iter().nth(option),
                        None => None,
                    }
                } else {
                    options.pop()
                }
            }
        };

        let entry = match entry {
            Some(entry) => match entry.get_credential() {
                Ok(entry) => Some(entry),
                Err(keyring_core::Error::NoEntry) => None,
                Err(err) => return Err(err.into()),
            },
            None => None,
        };

        match entry {
            Some(entry) => {
                let secret = entry.get_secret()?;
                let creds = serde_json::from_slice::<Credentials>(&secret)?;

                if creds.is_expired() {
                    *credentials = Some(creds);

                    self.auth_flow
                        .authenticate(credentials, http_client, server)
                        .map_err(Error::Inner)?;

                    if let Some(credentials) = credentials {
                        let secret = serde_json::to_vec(&credentials)?;

                        entry.set_secret(&secret)?;
                    } else {
                        entry.delete_credential()?;
                    }
                } else {
                    *credentials = Some(creds);
                }

                Ok(())
            }
            None => {
                self.auth_flow
                    .authenticate(credentials, http_client, server)
                    .map_err(Error::Inner)?;

                if let Some(credentials) = credentials {
                    let entry = store.build(&service, credentials.user(), None)?;
                    let secret = serde_json::to_vec(&credentials)?;

                    entry.set_secret(&secret)?;
                }

                Ok(())
            }
        }
    }
}
