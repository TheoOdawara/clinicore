use axum::Router;
use axum::http::{Method, StatusCode};
use axum::routing::get;
use clinicore_app::http::layers;
use clinicore_core::config::AppEnv;

use crate::support::{app, send, state};

#[tokio::test]
async fn every_answer_forbids_caching_errors_included() {
    for (method, path, status) in [
        (Method::GET, "/health", StatusCode::OK),
        (Method::GET, "/missing", StatusCode::NOT_FOUND),
        (Method::POST, "/health", StatusCode::METHOD_NOT_ALLOWED),
    ] {
        let answer = send(app(AppEnv::Test), method, path).await;

        assert_eq!(answer.status, status, "{path}");
        assert_eq!(answer.headers["cache-control"], "no-store", "{path}");
    }
}

#[tokio::test]
async fn a_panic_answer_forbids_caching_too() {
    let panicking = layers(
        Router::new().route("/panic", get(|| async { panic!("boom") })),
        state(),
    );

    let answer = send(panicking, Method::GET, "/panic").await;

    assert_eq!(answer.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(answer.headers["cache-control"], "no-store");
}
