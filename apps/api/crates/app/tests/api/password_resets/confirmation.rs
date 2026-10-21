use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::sessions::{Transport, revoked_ttl, sign_in};
use crate::support::{
    MOBILE, PASSWORD, app_with, count, fresh_email, json_body, post_json, register, request,
    request_from,
};

const PATH: &str = "/password-resets/confirmation";
const NEW_PASSWORD: &str = "Outra#Senha9";

async fn pending_reset(pool: &PgPool, email: &str, secret: &str, expires_in: &str) {
    sqlx::query(
        "INSERT INTO verifications (email, purpose, token_hash, expires_at)
        VALUES ($1, 'password_reset', encode(sha256($2::bytea), 'hex'), now() + $3::interval)",
    )
    .bind(email)
    .bind(secret)
    .bind(expires_in)
    .execute(pool)
    .await
    .expect("a pending reset");
}

async fn confirm(app: &axum::Router, secret: &str, password: &str) -> axum::response::Response {
    let body = json!({"token": secret, "newPassword": password});
    request(app.clone(), "POST", PATH, &[MOBILE], Some(&body)).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn resetting_the_password_revokes_every_session_at_once(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let web = Transport::Web.sign_in(&app, email).await;
    let mobile = Transport::Mobile.sign_in(&app, email).await;
    let used = "a".repeat(43);
    let sibling = "b".repeat(43);
    pending_reset(&pool, email, &used, "1 hour").await;
    pending_reset(&pool, email, &sibling, "1 hour").await;

    let response = confirm(&app, &used, NEW_PASSWORD).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
    for (transport, session) in [(Transport::Web, &web), (Transport::Mobile, &mobile)] {
        assert!(revoked_ttl(session.id()).await > 0, "{transport:?}");
        let read = transport
            .call(&app, "GET", "/sessions/current", &session.access)
            .await;
        assert_eq!(read.status(), StatusCode::UNAUTHORIZED, "{transport:?}");
    }
    let with_the_new = sign_in(&app, email, NEW_PASSWORD, &[]).await;
    let with_the_old = sign_in(&app, email, PASSWORD, &[]).await;
    assert_eq!(with_the_new.status(), StatusCode::CREATED);
    assert_eq!(with_the_old.status(), StatusCode::UNAUTHORIZED);

    for secret in [&used, &sibling] {
        let again = confirm(&app, secret, "Mais#Uma7").await;
        assert_eq!(again.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            json_body(again).await,
            json!({"type": "tag:clinicore.com.br,2026:invalid-token", "title": "Invalid token", "status": 400})
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_google_only_account_gains_the_password_sign_in(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    sqlx::query(
        "WITH created AS (
            INSERT INTO users (name, email, email_verified) VALUES ('Ana Souza', $1, true)
            RETURNING id
        )
        INSERT INTO accounts (user_id, provider, provider_account_id)
        SELECT id, 'google', 'google-subject' FROM created",
    )
    .bind(email)
    .execute(&pool)
    .await
    .expect("a google-only account");
    let requested = post_json(app.clone(), "/password-resets", &json!({"email": email})).await;
    assert_eq!(requested.status(), StatusCode::ACCEPTED);
    let issued = "SELECT count(*) FROM verifications WHERE purpose = 'password_reset'";
    assert_eq!(count(&pool, issued).await, 1);
    let secret = "c".repeat(43);
    pending_reset(&pool, email, &secret, "1 hour").await;

    let response = confirm(&app, &secret, PASSWORD).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let credential =
        "SELECT count(*) FROM accounts WHERE provider = 'credential' AND password_hash IS NOT NULL";
    let google = "SELECT count(*) FROM accounts WHERE provider = 'google'";
    assert_eq!(count(&pool, credential).await, 1);
    assert_eq!(count(&pool, google).await, 1);
    let signed_in = sign_in(&app, email, PASSWORD, &[]).await;
    assert_eq!(signed_in.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_weak_password_or_an_expired_link_keeps_the_password(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let live = "d".repeat(43);
    let expired = "e".repeat(43);
    pending_reset(&pool, email, &live, "1 hour").await;
    pending_reset(&pool, email, &expired, "-1 minute").await;

    for weak in ["sem_maiuscula#1", "SEM_DIGITO#a", "SemEspecial1", "Aa#1"] {
        let response = confirm(&app, &live, weak).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{weak}");
        assert_eq!(
            json_body(response).await["errors"],
            json!([{"pointer": "#/newPassword", "code": "weak_password"}]),
            "{weak}"
        );
    }
    let late = confirm(&app, &expired, NEW_PASSWORD).await;
    assert_eq!(late.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json_body(late).await["type"],
        "tag:clinicore.com.br,2026:invalid-token"
    );

    let signed_in = sign_in(&app, email, PASSWORD, &[]).await;
    assert_eq!(signed_in.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn confirmations_stop_at_the_network_ceiling_and_spare_every_other_network(pool: PgPool) {
    let app = app_with(&pool);
    let url = std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run");
    let mut connection = redis::Client::open(url)
        .expect("a redis url")
        .get_multiplexed_async_connection()
        .await
        .expect("a redis connection");
    let key = "rate:password-reset-confirmation-network:203.0.113.0/24";
    let _: () = redis::cmd("SET")
        .arg(key)
        .arg(300)
        .arg("EX")
        .arg(60)
        .query_async(&mut connection)
        .await
        .expect("a network ceiling already reached");
    let host = 1 + SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after the epoch")
        .subsec_nanos()
        % 254;

    let body = json!({"token": "f".repeat(43), "newPassword": NEW_PASSWORD});
    let mut statuses = Vec::new();
    for network in ["203.0.113", "198.51.100"] {
        let client = format!("{network}.{host}:4000")
            .parse()
            .expect("a socket address");
        let response =
            request_from(client, app.clone(), "POST", PATH, &[MOBILE], Some(&body)).await;
        statuses.push(response.status());
    }

    let _: () = redis::cmd("DEL")
        .arg(key)
        .query_async(&mut connection)
        .await
        .expect("the ceiling cleared");
    assert_eq!(
        statuses,
        [StatusCode::TOO_MANY_REQUESTS, StatusCode::BAD_REQUEST]
    );
}
