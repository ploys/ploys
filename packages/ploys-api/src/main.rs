mod github;
mod state;

use std::sync::Arc;

use anyhow::Context;
use axum::routing::post;
use axum::{Extension, Router};
use shuttle_runtime::SecretStore;

use self::github::webhook::secret::WebhookSecret;
use self::state::AppState;

#[shuttle_runtime::main]
async fn axum(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let state = AppState {
        github_app_client_id: Arc::from(
            secret_store
                .get("GITHUB_APP_CLIENT_ID")
                .context("Missing GITHUB_APP_CLIENT_ID secret.")?,
        ),
        github_app_private_key: Arc::from(
            secret_store
                .get("GITHUB_APP_PRIVATE_KEY")
                .context("Missing GITHUB_APP_PRIVATE_KEY secret.")?,
        ),
    };

    let router = Router::new()
        .route(
            "/github/webhook",
            post(self::github::webhook::receive).layer(Extension(WebhookSecret::from_store(
                &secret_store,
                "GITHUB_APP_WEBHOOK_SECRET",
            )?)),
        )
        .with_state(state);

    Ok(router.into())
}
