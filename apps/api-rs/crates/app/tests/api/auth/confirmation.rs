use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::{MOBILE, app_with, count, register};
use crate::support::{fresh_client, json_body, request, set_cookie};

const EMAIL: &str = "ana@example.com";

async fn pending_link(pool: &PgPool, secret: &str, expires_in: &str) {
    sqlx::query(
        "INSERT INTO verifications (email, purpose, token_hash, expires_at)
        VALUES ($1, 'email_verification', encode(sha256($2::bytea), 'hex'), now() + $3::interval)",
    )
    .bind(EMAIL)
    .bind(secret)
    .bind(expires_in)
    .execute(pool)
    .await
    .expect("a pending link");
}

async fn confirm(app: &axum::Router, secret: &str) -> axum::response::Response {
    let body = json!({"token": secret});
    request(
        app.clone(),
        "POST",
        "/email-verifications/confirmation",
        &[MOBILE],
        Some(&body),
        fresh_client(),
    )
    .await
}

#[sqlx::test(migrations = "../../migrations")]
async fn confirming_the_email_signs_in_once_with_cookies(pool: PgPool) {
    let app = app_with(&pool);
    register(&app, &pool, EMAIL, false).await;
    let first = "a".repeat(43);
    let second = "b".repeat(43);
    pending_link(&pool, &first, "1 hour").await;
    pending_link(&pool, &second, "1 hour").await;

    let response = confirm(&app, &first).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(set_cookie(&response, "clinicore_access").is_some());
    assert!(set_cookie(&response, "clinicore_refresh").is_some());
    let verified = "SELECT count(*) FROM users WHERE email_verified";
    let pending = "SELECT count(*) FROM verifications WHERE consumed_at IS NULL";
    let web_sessions = "SELECT count(*) FROM sessions WHERE client = 'web'";
    assert_eq!(count(&pool, verified).await, 1);
    assert_eq!(count(&pool, pending).await, 0);
    assert_eq!(count(&pool, web_sessions).await, 1);

    for secret in [&first, &second] {
        let again = confirm(&app, secret).await;
        assert_eq!(again.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            json_body(again).await,
            json!({"type": "tag:clinicore.com.br,2026:invalid-token", "title": "Invalid token", "status": 400})
        );
    }
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_or_expired_link_is_refused_with_its_code(pool: PgPool) {
    let app = app_with(&pool);
    register(&app, &pool, EMAIL, false).await;
    let expired = "c".repeat(43);
    pending_link(&pool, &expired, "-1 minute").await;

    for (secret, slug, title) in [
        ("d".repeat(43), "invalid-token", "Invalid token"),
        (expired, "token-expired", "Token expired"),
    ] {
        let response = confirm(&app, &secret).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{slug}");
        assert!(response.headers().get("set-cookie").is_none());
        assert_eq!(
            json_body(response).await,
            json!({"type": format!("tag:clinicore.com.br,2026:{slug}"), "title": title, "status": 400})
        );
    }
    assert_eq!(
        count(&pool, "SELECT count(*) FROM users WHERE email_verified").await,
        0
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
}
