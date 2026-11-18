use axum::Router;
use axum::http::{Method, StatusCode};
use axum::routing::get;
use clinicore_app::http::layers;
use clinicore_core::config::AppEnv;
use serde_json::json;

use crate::support::{app, send, state};

#[tokio::test]
async fn an_unknown_path_answers_a_not_found_problem() {
    let answer = send(app(AppEnv::Test), Method::GET, "/missing").await;

    assert_eq!(answer.status, StatusCode::NOT_FOUND);
    assert_eq!(answer.headers["content-type"], "application/problem+json");
    assert_eq!(
        answer.json(),
        json!({ "type": "about:blank", "title": "Not Found", "status": 404 })
    );
}

#[tokio::test]
async fn an_unaccepted_method_answers_a_method_not_allowed_problem() {
    let answer = send(app(AppEnv::Test), Method::POST, "/health").await;

    assert_eq!(answer.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(answer.headers["content-type"], "application/problem+json");
    assert_eq!(
        answer.json(),
        json!({ "type": "about:blank", "title": "Method Not Allowed", "status": 405 })
    );
}

#[tokio::test]
async fn a_panicking_handler_answers_an_internal_problem_without_detail() {
    for route in [
        get(|| async { panic!("database password is hunter2") }),
        get(|| async { panic!("{}", String::from("owned message")) }),
    ] {
        let panicking = layers(Router::new().route("/panic", route), state());

        let answer = send(panicking, Method::GET, "/panic").await;

        assert_eq!(answer.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(answer.headers["content-type"], "application/problem+json");
        assert_eq!(
            answer.json(),
            json!({ "type": "about:blank", "title": "Internal Server Error", "status": 500 })
        );
    }
}
