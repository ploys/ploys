use std::sync::Arc;

use anyhow::Error;
use axum::routing::post;
use axum::{Extension, Router};
use clap::Args;
use tokio::net::TcpListener;

use crate::github::webhook::secret::WebhookSecret;
use crate::state::AppState;

/// The serve command.
#[derive(Args)]
pub struct Serve {
    /// The API address.
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: std::net::SocketAddr,
    /// The GitHub App Client ID.
    #[arg(long, env = "GITHUB_APP_CLIENT_ID", hide_env_values = true)]
    client_id: String,
    /// The GitHub App Private Key
    #[arg(long, env = "GITHUB_APP_PRIVATE_KEY", hide_env_values = true)]
    private_key: String,
    /// The GitHub Webhook Secret.
    #[arg(long, env = "GITHUB_APP_WEBHOOK_SECRET", hide_env_values = true)]
    webhook_secret: String,
}

impl Serve {
    /// Executes the serve command.
    pub async fn exec(self) -> Result<(), Error> {
        let state = AppState {
            github_app_client_id: Arc::from(self.client_id),
            github_app_private_key: Arc::from(self.private_key),
        };

        let router = Router::new()
            .route(
                "/github/webhook",
                post(crate::github::webhook::receive)
                    .layer(Extension(WebhookSecret::new(self.webhook_secret))),
            )
            .with_state(state);

        let listener = TcpListener::bind(self.addr).await?;

        axum::serve(listener, router).await?;

        Ok(())
    }
}
