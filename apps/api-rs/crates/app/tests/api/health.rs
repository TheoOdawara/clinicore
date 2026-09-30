use crate::support::{config_with, content_type, json_body, request, send, text_body};
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn health_answers_ok() {
    let response = send(crate::support::app(&config_with(&[])), "GET", "/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(text_body(response).await, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn an_unsupported_method_on_health_answers_404_even_from_a_foreign_origin() {
    let response = request(
        crate::support::app(&config_with(&[])),
        "POST",
        "/health",
        &[("origin", "https://evil.example"), ("cookie", "session=1")],
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(content_type(&response).starts_with("application/problem+json"));
    assert_eq!(
        json_body(response).await,
        json!({"type": "about:blank", "title": "Not Found", "status": 404})
    );
}
