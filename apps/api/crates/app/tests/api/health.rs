use axum::http::{Method, StatusCode};
use clinicore_core::config::AppEnv;

use crate::support::{app, send};

#[tokio::test]
async fn health_answers_ok_without_touching_the_database() {
    let answer = send(app(AppEnv::Test), Method::GET, "/health").await;

    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.body, br#"{"status":"ok"}"#);
}
