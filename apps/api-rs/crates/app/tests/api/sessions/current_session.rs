use axum::http::StatusCode;
use clinicore_app::AppState;
use clinicore_core::mail::Mailer;
use clinicore_core::redis::Redis;
use lettre::transport::stub::AsyncStubTransport;
use serde_json::json;
use sqlx::PgPool;

use super::{Transport, WEB_ORIGIN, revoked_ttl, session_exists, sign_in};
use crate::support::{
    MOBILE, PASSWORD, app_with, config_with, fresh_client, fresh_email, json_body, register,
    request, send, set_cookie,
};

#[sqlx::test(migrations = "../../migrations")]
async fn sign_out_revokes_the_session_at_once_on_each_transport(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;

    for transport in Transport::BOTH {
        let session = transport.sign_in(&app, email).await;

        let response = transport
            .call(&app, "DELETE", "/sessions/current", &session.access)
            .await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT, "{transport:?}");
        match transport {
            Transport::Web => {
                for name in ["clinicore_access", "clinicore_refresh"] {
                    let cleared = set_cookie(&response, name).expect("a cleared cookie");
                    assert!(cleared.contains("Max-Age=0"), "{cleared}");
                }
            }
            Transport::Mobile => assert!(response.headers().get("set-cookie").is_none()),
        }
        assert!(!session_exists(&pool, session.id()).await);
        let ttl = revoked_ttl(session.id()).await;
        assert!((1..=900).contains(&ttl), "{ttl}");

        for method in ["GET", "DELETE"] {
            let again = transport
                .call(&app, method, "/sessions/current", &session.access)
                .await;
            assert_eq!(
                again.status(),
                StatusCode::UNAUTHORIZED,
                "{transport:?} {method}"
            );
            if let (Transport::Web, "DELETE") = (transport, method) {
                let cleared = set_cookie(&again, "clinicore_refresh").expect("a cleared cookie");
                assert!(cleared.contains("Max-Age=0"), "{cleared}");
            }
            assert_eq!(
                json_body(again).await,
                json!({"type": "tag:clinicore.com.br,2026:invalid-session", "title": "Invalid session", "status": 401})
            );
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn one_transport_never_authenticates_through_the_other(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let web = Transport::Web.sign_in(&app, email).await;
    let mobile = Transport::Mobile.sign_in(&app, email).await;
    let mobile_cookie = format!("clinicore_access={}", mobile.access);
    let web_bearer = format!("Bearer {}", web.access);
    let mobile_bearer = format!("Bearer {}", mobile.access);

    let crossings: [&[(&str, &str)]; 3] = [
        &[("cookie", &mobile_cookie)],
        &[MOBILE, ("authorization", &web_bearer)],
        &[("authorization", &mobile_bearer)],
    ];
    for headers in crossings {
        let response = request(
            app.clone(),
            "GET",
            "/sessions/current",
            headers,
            None,
            fresh_client(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{headers:?}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_origin_guard_lets_the_app_through_and_holds_cross_site_requests(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let session = Transport::Web.sign_in(&app, email).await;
    let access_cookie = format!("clinicore_access={}", session.access);
    let refresh_cookie = format!("clinicore_refresh={}", session.refresh);

    for origin in [
        "http://evil.example",
        "https://clinicore.com.br.evil.example",
    ] {
        let response = sign_in(&app, email, PASSWORD, &[("origin", origin)], fresh_client()).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{origin}");
        assert_eq!(
            json_body(response).await,
            json!({"type": "tag:clinicore.com.br,2026:invalid-origin", "title": "Invalid origin", "status": 403})
        );
    }
    let refused = [
        ("DELETE", "/sessions/current", access_cookie.as_str()),
        ("POST", "/sessions/current/tokens", refresh_cookie.as_str()),
    ];
    for (method, path, cookie) in refused {
        let response = request(
            app.clone(),
            method,
            path,
            &[("cookie", cookie)],
            None,
            fresh_client(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{method} {path}");
    }
    assert!(session_exists(&pool, session.id()).await);

    let read = request(
        app.clone(),
        "GET",
        "/sessions/current",
        &[("cookie", &access_cookie)],
        None,
        fresh_client(),
    )
    .await;
    assert_eq!(read.status(), StatusCode::OK);
    let app_sign_in = sign_in(&app, email, PASSWORD, &[MOBILE], fresh_client()).await;
    assert_eq!(app_sign_in.status(), StatusCode::CREATED);

    let allowed = request(
        app.clone(),
        "GET",
        "/health",
        &[WEB_ORIGIN],
        None,
        fresh_client(),
    )
    .await;
    assert_eq!(
        allowed.headers()["access-control-allow-origin"],
        "http://localhost:3000"
    );
    assert_eq!(
        allowed.headers()["access-control-allow-credentials"],
        "true"
    );
    let foreign = request(
        app.clone(),
        "GET",
        "/health",
        &[("origin", "http://evil.example")],
        None,
        fresh_client(),
    )
    .await;
    assert!(
        foreign
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_redis_outage_closes_the_door_instead_of_opening_it(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let session = Transport::Web.sign_in(&app, email).await;

    let mut config = config_with(&[]);
    config.redis_url = "redis://127.0.0.1:1".to_string();
    let state = AppState::new(
        &config,
        pool.clone(),
        Redis::connect_lazy(&config).expect("a lazy redis"),
        Mailer::stub(&config, AsyncStubTransport::new_ok()).expect("a valid sender"),
    )
    .expect("an unmatchable hash");
    let outage = clinicore_app::app(&config, state);

    for method in ["GET", "DELETE"] {
        let response = Transport::Web
            .call(&outage, method, "/sessions/current", &session.access)
            .await;
        assert_eq!(
            response.status(),
            StatusCode::SERVICE_UNAVAILABLE,
            "{method}"
        );
    }
    assert!(session_exists(&pool, session.id()).await);
    assert_eq!(
        send(outage, "GET", "/health").await.status(),
        StatusCode::OK
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_session_answers_at_most_a_hundred_requests_in_ten_seconds_whatever_the_address(
    pool: PgPool,
) {
    let email = fresh_email();
    let app = app_with(&pool);
    register(&app, &pool, &email, true).await;
    let session = Transport::Mobile.sign_in(&app, &email).await;

    for attempt in 1..=100 {
        let response = Transport::Mobile
            .call(&app, "GET", "/sessions/current", &session.access)
            .await;
        assert_eq!(response.status(), StatusCode::OK, "attempt {attempt}");
    }
    let response = Transport::Mobile
        .call(&app, "GET", "/sessions/current", &session.access)
        .await;

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}
