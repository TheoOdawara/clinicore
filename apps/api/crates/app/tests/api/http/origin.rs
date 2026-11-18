use axum::Router;
use axum::http::{Method, StatusCode};
use axum::routing::post;
use clinicore_app::http::layers;
use serde_json::json;

use crate::support::{Answer, WEB_ORIGIN, send_with, state};

fn probe() -> Router {
    let route = post(|| async { StatusCode::NO_CONTENT })
        .put(|| async { StatusCode::NO_CONTENT })
        .patch(|| async { StatusCode::NO_CONTENT })
        .delete(|| async { StatusCode::NO_CONTENT })
        .get(|| async { StatusCode::NO_CONTENT });
    layers(Router::new().route("/probe", route), state())
}

async fn probe_with(method: Method, headers: &[(&str, &str)]) -> Answer {
    send_with(probe(), method, "/probe", headers).await
}

fn assert_problem(answer: &Answer, status: StatusCode, slug: &str, title: &str) {
    assert_eq!(answer.status, status);
    assert_eq!(answer.headers["content-type"], "application/problem+json");
    assert_eq!(
        answer.json(),
        json!({
            "type": format!("tag:clinicore.com.br,2026:{slug}"),
            "title": title,
            "status": status.as_u16(),
        })
    );
}

#[tokio::test]
async fn a_listed_origin_and_a_request_without_origin_or_cookie_change_state() {
    for method in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
        for headers in [
            &[("origin", WEB_ORIGIN)][..],
            &[
                ("origin", "https://app.clinicore.com.br"),
                ("cookie", "a=b"),
            ][..],
            &[][..],
            &[("clinicore-client", "mobile")][..],
        ] {
            let answer = probe_with(method.clone(), headers).await;

            assert_eq!(
                answer.status,
                StatusCode::NO_CONTENT,
                "{method} {headers:?}"
            );
        }
    }
}

#[tokio::test]
async fn an_unlisted_origin_or_a_cookie_without_origin_is_refused() {
    for headers in [
        &[("origin", "https://evil.example.com")][..],
        &[("origin", "https://app.clinicore.com.br.evil.example.com")][..],
        &[("origin", "http://localhost:3000/")][..],
        &[("origin", "null")][..],
        &[("cookie", "clinicore_access=token")][..],
        &[("clinicore-client", "mobile"), ("cookie", "a=b")][..],
    ] {
        let answer = probe_with(Method::POST, headers).await;

        assert_problem(
            &answer,
            StatusCode::FORBIDDEN,
            "invalid-origin",
            "Invalid origin",
        );
    }
}

#[tokio::test]
async fn a_safe_method_is_not_guarded() {
    let answer = probe_with(
        Method::GET,
        &[("origin", "https://evil.example.com"), ("cookie", "a=b")],
    )
    .await;

    assert_eq!(answer.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn an_unknown_client_is_refused_before_the_origin() {
    for method in [Method::GET, Method::POST] {
        let answer = probe_with(
            method,
            &[
                ("clinicore-client", "desktop"),
                ("origin", "https://evil.example.com"),
            ],
        )
        .await;

        assert_problem(
            &answer,
            StatusCode::BAD_REQUEST,
            "invalid-client",
            "Invalid client",
        );
    }
}

#[tokio::test]
async fn an_unknown_path_stays_not_found_whatever_the_origin() {
    let answer = send_with(
        probe(),
        Method::POST,
        "/missing",
        &[("origin", "https://evil.example.com")],
    )
    .await;

    assert_eq!(answer.status, StatusCode::NOT_FOUND);
}
