mod github;

use axum::routing::post;
use axum::{Extension, Router};
use shuttle_runtime::SecretStore;

use self::github::webhook::secret::WebhookSecret;

#[shuttle_runtime::main]
async fn axum(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route(
        "/github/webhook",
        post(self::github::webhook::receive).layer(Extension(WebhookSecret::from_store(
            &secret_store,
            "GITHUB_APP_WEBHOOK_SECRET",
        )?)),
    );

    Ok(router.into())
}
