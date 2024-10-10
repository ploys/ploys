use std::sync::Arc;

/// The application state.
#[derive(Clone)]
pub struct AppState {
    pub github_app_client_id: Arc<str>,
    pub github_app_private_key: Arc<str>,
}
