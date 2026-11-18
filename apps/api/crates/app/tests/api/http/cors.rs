use axum::http::{Method, StatusCode};
use clinicore_core::config::AppEnv;

use crate::support::{WEB_ORIGIN, app, send_with};

#[tokio::test]
async fn a_preflight_from_a_listed_origin_is_allowed_with_credentials() {
    let answer = send_with(
        app(AppEnv::Test),
        Method::OPTIONS,
        "/health",
        &[
            ("origin", WEB_ORIGIN),
            ("access-control-request-method", "POST"),
            (
                "access-control-request-headers",
                "content-type,authorization,clinicore-client",
            ),
        ],
    )
    .await;

    assert_eq!(answer.status, StatusCode::OK);
    let header = |name: &str| answer.headers[name].to_str().expect("ascii").to_owned();
    assert_eq!(header("access-control-allow-origin"), WEB_ORIGIN);
    assert_eq!(header("access-control-allow-credentials"), "true");
    assert_eq!(
        header("access-control-allow-methods"),
        "GET,POST,PUT,PATCH,DELETE"
    );
    assert_eq!(
        header("access-control-allow-headers"),
        "content-type,authorization,clinicore-client"
    );
    assert_eq!(header("access-control-max-age"), "600");
}

#[tokio::test]
async fn an_unlisted_origin_gets_no_allow_origin() {
    for origin in [
        "https://evil.example.com",
        "https://app.clinicore.com.br.evil.example.com",
    ] {
        let answer = send_with(
            app(AppEnv::Test),
            Method::OPTIONS,
            "/health",
            &[
                ("origin", origin),
                ("access-control-request-method", "POST"),
            ],
        )
        .await;

        assert!(
            !answer.headers.contains_key("access-control-allow-origin"),
            "{origin}"
        );
    }
}

#[tokio::test]
async fn a_simple_request_from_a_listed_origin_carries_the_allow_headers() {
    let answer = send_with(
        app(AppEnv::Test),
        Method::GET,
        "/health",
        &[("origin", WEB_ORIGIN)],
    )
    .await;

    assert_eq!(answer.headers["access-control-allow-origin"], WEB_ORIGIN);
    assert_eq!(answer.headers["access-control-allow-credentials"], "true");
}
