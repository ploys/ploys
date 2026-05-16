mod login;
mod status;

use std::sync::Arc;

use anyhow::Error;
use clap::{Args, Subcommand};
use keyring_core::CredentialStore;

use self::login::Login;
use self::status::Status;

/// The `auth` command.
#[derive(Args)]
pub struct Auth {
    #[command(subcommand)]
    command: Command,
}

impl Auth {
    /// Executes the `auth` command.
    pub fn exec(self) -> Result<(), Error> {
        match self.command {
            Command::Login(login) => login.exec(),
            Command::Status(status) => status.exec(),
        }
    }
}

/// The `auth` subcommands.
#[derive(Subcommand)]
enum Command {
    /// Log in to GitHub.
    Login(Login),
    /// View authentication status.
    Status(Status),
}

/// Initialises the platform keyring store.
pub fn init_keyring() -> Result<Arc<CredentialStore>, Error> {
    #[cfg(target_os = "macos")]
    {
        Ok(apple_native_keyring_store::keychain::Store::new()?)
    }

    #[cfg(target_os = "linux")]
    {
        Ok(dbus_secret_service_keyring_store::Store::new()?)
    }

    #[cfg(target_os = "windows")]
    {
        Ok(windows_native_keyring_store::Store::new()?)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}
