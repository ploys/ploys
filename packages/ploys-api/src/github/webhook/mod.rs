mod header;
mod payload;
pub mod secret;

use axum::http::StatusCode;

use self::payload::Payload;

/// Receives the GitHub webhook event payload.
pub async fn receive(payload: Payload) -> StatusCode {
    println!("Event: {}", payload.event);
    println!("Payload: {:#}", payload.value);

    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use axum::routing::post;
    use axum::{Extension, Router};
    use hmac::{Hmac, Mac};
    use serde_json::{json, Value};
    use sha2::Sha256;
    use tower_service::Service;

    use super::secret::WebhookSecret;

    fn router() -> Router {
        Router::new().route(
            "/github/webhook",
            post(super::receive).layer(Extension(WebhookSecret {
                value: String::from("super_secret"),
            })),
        )
    }

    fn payload() -> Value {
        json!({
            "action": "opened",
            "issue": {
              "url": "https://api.github.com/repos/octocat/Hello-World/issues/1347",
              "number": 1347
            },
            "repository" : {
              "id": 1296269,
              "full_name": "octocat/Hello-World",
              "owner": {
                  "login": "octocat",
                  "id": 1
              }
            },
            "sender": {
              "login": "octocat",
              "id": 1
            }
        })
    }

    #[tokio::test]
    async fn test_webhook_endpoint_valid_signature() {
        let mut router = router();
        let payload = serde_json::to_string(&payload()).unwrap();
        let mut hmac = Hmac::<Sha256>::new_from_slice(b"super_secret").unwrap();

        hmac.update(payload.as_bytes());

        let digest = hmac.finalize().into_bytes();
        let hex = hex::encode(digest);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/github/webhook")
            .header("Content-Type", "application/json")
            .header("X-GitHub-Event", "issues")
            .header("X-Hub-Signature-256", format!("sha256={hex}"))
            .body(Body::from(payload))
            .unwrap();

        let response = router.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_webhook_endpoint_invalid_signature() {
        let mut router = router();
        let payload = serde_json::to_string(&payload()).unwrap();
        let mut hmac = Hmac::<Sha256>::new_from_slice(b"not_super_secret").unwrap();

        hmac.update(payload.as_bytes());

        let digest = hmac.finalize().into_bytes();
        let hex = hex::encode(digest);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/github/webhook")
            .header("Content-Type", "application/json")
            .header("X-GitHub-Event", "issues")
            .header("X-Hub-Signature-256", format!("sha256={hex}"))
            .body(Body::from(payload))
            .unwrap();

        let response = router.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
