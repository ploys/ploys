mod github;

use axum::routing::post;
use axum::Router;

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/github/webhook", post(self::github::webhook::receive));

    Ok(router.into())
}
