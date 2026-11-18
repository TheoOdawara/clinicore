use axum::http::Method;
use clinicore_core::config::AppEnv;

use crate::support::{Captured, app, send};

async fn log_of(method: Method, path: &str) -> String {
    let captured = Captured::default();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_current_span(true)
        .with_writer(captured.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    send(app(AppEnv::Test), method, path).await;

    captured.text()
}

#[tokio::test]
async fn a_request_is_logged_with_method_path_and_status() {
    let log = log_of(Method::GET, "/missing?token=secret-value").await;

    assert!(log.contains(r#""status":404"#), "{log}");
    assert!(log.contains(r#""method":"GET""#), "{log}");
    assert!(log.contains(r#""path":"/missing""#), "{log}");
    assert!(
        !log.contains("secret-value"),
        "the query string is not logged"
    );
}

#[tokio::test]
async fn health_is_not_logged() {
    let log = log_of(Method::GET, "/health").await;

    assert!(log.is_empty(), "{log}");
}
