use axum::http::StatusCode;
use lettre::transport::stub::AsyncStubTransport;
use serde_json::json;
use sqlx::PgPool;

use super::{MOBILE, app_with, count, fresh_email, register};
use crate::support::{config_with, fresh_client, json_body, request, state_on_redis};

async fn pending_link(pool: &PgPool, email: &str, secret: &str, expires_in: &str) {
    sqlx::query(
        "INSERT INTO verifications (email, purpose, token_hash, expires_at)
        VALUES ($1, 'email_verification', encode(sha256($2::bytea), 'hex'), now() + $3::interval)",
    )
    .bind(email)
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
async fn confirming_the_email_verifies_it_once_and_opens_no_session(pool: PgPool) {
    let email = fresh_email();
    let app = app_with(&pool);
    register(&app, &pool, &email, false).await;
    let first = "a".repeat(43);
    let second = "b".repeat(43);
    pending_link(&pool, &email, &first, "1 hour").await;
    pending_link(&pool, &email, &second, "1 hour").await;

    let response = confirm(&app, &first).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(response.headers().get("set-cookie").is_none());
    let verified = "SELECT count(*) FROM users WHERE email_verified";
    let pending = "SELECT count(*) FROM verifications WHERE consumed_at IS NULL";
    assert_eq!(count(&pool, verified).await, 1);
    assert_eq!(count(&pool, pending).await, 0);

    for secret in [&first, &second] {
        let again = confirm(&app, secret).await;
        assert_eq!(again.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            json_body(again).await,
            json!({"type": "tag:clinicore.com.br,2026:invalid-token", "title": "Invalid token", "status": 400})
        );
    }
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_links_of_one_address_confirmed_together_verify_it_once(pool: PgPool) {
    let app = app_with(&pool);

    for round in 0..10 {
        let email = fresh_email();
        register(&app, &pool, &email, false).await;
        let first = format!("f{round:0>42}");
        let second = format!("s{round:0>42}");
        pending_link(&pool, &email, &first, "1 hour").await;
        pending_link(&pool, &email, &second, "1 hour").await;

        let (one, other) = tokio::join!(confirm(&app, &first), confirm(&app, &second));

        let mut statuses = [one.status().as_u16(), other.status().as_u16()];
        statuses.sort();
        assert_eq!(statuses, [204, 400], "round {round}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_or_expired_link_is_refused_with_its_code(pool: PgPool) {
    let email = fresh_email();
    let app = app_with(&pool);
    register(&app, &pool, &email, false).await;
    let expired = "c".repeat(43);
    pending_link(&pool, &email, &expired, "-1 minute").await;

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
}

#[sqlx::test(migrations = "../../migrations")]
async fn confirmations_stop_at_the_total_ceiling_whatever_the_address(pool: PgPool) {
    let redis_url = format!(
        "{}/15",
        std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run")
    );
    let config = config_with(&[]);
    let app = clinicore_app::app(
        &config,
        state_on_redis(
            &config,
            pool.clone(),
            AsyncStubTransport::new_ok(),
            &redis_url,
        ),
    );
    let mut connection = redis::Client::open(redis_url)
        .expect("a redis url")
        .get_multiplexed_async_connection()
        .await
        .expect("a redis connection");
    let key = "rate:email-confirmation-total:all";
    let _: () = redis::cmd("SET")
        .arg(key)
        .arg(300)
        .arg("EX")
        .arg(60)
        .query_async(&mut connection)
        .await
        .expect("a ceiling already reached");

    let response = confirm(&app, &"e".repeat(43)).await;

    let _: () = redis::cmd("DEL")
        .arg(key)
        .query_async(&mut connection)
        .await
        .expect("the ceiling cleared");
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}
