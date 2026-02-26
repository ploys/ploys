use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::state::AppState;

pub mod webhook;

/// Gets the application information.
pub async fn get(state: State<AppState>) -> Json<AppInfo> {
    Json(AppInfo {
        client_id: state.github_app_client_id.to_string(),
    })
}

#[derive(Serialize)]
pub struct AppInfo {
    client_id: String,
}
