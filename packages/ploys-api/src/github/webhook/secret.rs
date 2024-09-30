use anyhow::{anyhow, Error};
use shuttle_runtime::SecretStore;

#[derive(Clone)]
pub struct WebhookSecret {
    pub value: String,
}

impl WebhookSecret {
    /// Gets the secret from the secret store.
    pub fn from_store(store: &SecretStore, name: &str) -> Result<Self, Error> {
        match store.get(name) {
            Some(value) => Ok(Self { value }),
            None => Err(anyhow!("Missing GitHub App webhook secret.")),
        }
    }
}
