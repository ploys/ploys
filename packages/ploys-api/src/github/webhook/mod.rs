mod header;
mod payload;

use axum::http::StatusCode;

use self::payload::Payload;

/// Receives the GitHub webhook event payload.
pub async fn receive(payload: Payload) -> StatusCode {
    println!("Event: {}", payload.event);
    println!("Payload: {:#}", payload.value);

    StatusCode::OK
}
