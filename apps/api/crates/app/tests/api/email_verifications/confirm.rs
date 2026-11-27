use axum::http::{StatusCode, header};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::support::{Answer, app_on, post_json, token_in};

async fn sign_up(pool: &PgPool) {
    let body =
        json!({ "name": "Ana Souza", "email": "ana@exemplo.com", "password": "Clinica#2026" });
    let answer = post_json(app_on(pool.clone()), "/users", &body, &[]).await;
    assert_eq!(answer.status, StatusCode::ACCEPTED);
}

async fn latest_token(pool: &PgPool) -> String {
    let text: String = sqlx::query_scalar(
        "SELECT text_body FROM email_outbox WHERE email = 'ana@exemplo.com'
         ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .expect("an email is queued");
    token_in(&text)
}

async fn confirm(pool: &PgPool, token: Value) -> Answer {
    post_json(
        app_on(pool.clone()),
        "/email-verifications/confirmation",
        &json!({ "token": token }),
        &[],
    )
    .await
}

fn assert_problem(answer: &Answer, slug: &str, title: &str) {
    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        answer.json(),
        json!({ "type": format!("tag:clinicore.com.br,2026:{slug}"), "title": title, "status": 400 })
    );
}

async fn verified(pool: &PgPool) -> bool {
    sqlx::query_scalar("SELECT email_verified FROM users WHERE email = 'ana@exemplo.com'")
        .fetch_one(pool)
        .await
        .expect("the user exists")
}

async fn consumed(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM verifications WHERE consumed_at IS NOT NULL")
        .fetch_one(pool)
        .await
        .expect("the verifications are readable")
}

#[sqlx::test(migrations = "../../migrations")]
async fn confirming_verifies_the_email_consumes_every_pending_link_and_opens_no_session(
    pool: PgPool,
) {
    sign_up(&pool).await;
    sqlx::query("UPDATE email_dispatches SET created_at = now() - interval '2 minutes'")
        .execute(&pool)
        .await
        .expect("the dispatch moves back");
    let answer = post_json(
        app_on(pool.clone()),
        "/email-verifications",
        &json!({ "email": "ana@exemplo.com" }),
        &[],
    )
    .await;
    assert_eq!(answer.status, StatusCode::ACCEPTED);
    let token = latest_token(&pool).await;

    let answer = confirm(&pool, json!(token)).await;

    assert_eq!(answer.status, StatusCode::NO_CONTENT);
    assert!(answer.body.is_empty());
    assert!(!answer.headers.contains_key(header::SET_COOKIE));
    assert_eq!(answer.headers[header::CACHE_CONTROL], "no-store");
    assert!(verified(&pool).await);
    assert_eq!(consumed(&pool).await, 2);

    let again = confirm(&pool, json!(token)).await;

    assert_problem(&again, "invalid-token", "Invalid token");
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_simultaneous_confirmations_of_one_token_verify_once(pool: PgPool) {
    sign_up(&pool).await;
    let token = json!(latest_token(&pool).await);

    let (first, second) = tokio::join!(confirm(&pool, token.clone()), confirm(&pool, token));

    let mut statuses = [first.status, second.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::NO_CONTENT, StatusCode::BAD_REQUEST]);
    assert!(verified(&pool).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_expired_or_unknown_token_is_refused_without_writing(pool: PgPool) {
    sign_up(&pool).await;
    let token = latest_token(&pool).await;
    sqlx::query(
        "UPDATE verifications
         SET created_at = now() - interval '61 minutes', expires_at = now() - interval '1 minute'",
    )
    .execute(&pool)
    .await
    .expect("the verification moves back");

    let expired = confirm(&pool, json!(token)).await;

    assert_problem(&expired, "token-expired", "Token expired");
    assert!(!verified(&pool).await);
    assert_eq!(consumed(&pool).await, 0);

    let unknown = confirm(&pool, json!("a".repeat(43))).await;

    assert_problem(&unknown, "invalid-token", "Invalid token");

    for malformed in [json!("a".repeat(42)), json!(43)] {
        let answer = confirm(&pool, malformed).await;
        assert_eq!(answer.status, StatusCode::BAD_REQUEST);
        assert_eq!(
            answer.json()["errors"],
            json!([{ "pointer": "#/token", "code": "invalid" }])
        );
    }
    assert!(!verified(&pool).await);
}
