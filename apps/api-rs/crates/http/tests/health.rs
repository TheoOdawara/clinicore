mod common;

use axum::http::StatusCode;
use common::{config_with, content_type, json_body, send, send_with_headers, text_body};
use serde_json::json;

#[tokio::test]
async fn health_answers_ok_with_the_terminus_body() {
    let response = send(api_http::app(&config_with(&[])), "GET", "/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        text_body(response).await,
        r#"{"status":"ok","info":{},"error":{},"details":{}}"#
    );
}

#[tokio::test]
async fn an_unsupported_method_on_health_answers_404_even_from_a_foreign_origin() {
    let response = send_with_headers(
        api_http::app(&config_with(&[])),
        "POST",
        "/health",
        &[("origin", "https://evil.example"), ("cookie", "session=1")],
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(content_type(&response).starts_with("application/problem+json"));
    assert_eq!(
        json_body(response).await,
        json!({"type": "about:blank", "title": "Not Found", "status": 404})
    );
}
