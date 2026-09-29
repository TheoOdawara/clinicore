use std::time::Duration;

use crate::support::{
    CapturedLog, config_with, eventually, fresh_client, post_json, services, text_body,
};
use axum::http::StatusCode;
use lettre::transport::stub::AsyncStubTransport;
use serde_json::json;
use sqlx::PgPool;

const EMAIL: &str = "ana@exemplo.com";

fn sign_up_body(email: &str) -> serde_json::Value {
    json!({"name": "Ana", "email": email, "password": "Clinica#2026"})
}

async fn count(pool: &PgPool, sql: &'static str, email: &str) -> i64 {
    sqlx::query_scalar(sql)
        .bind(email)
        .fetch_one(pool)
        .await
        .expect("a count")
}

async fn assert_accepted(response: axum::response::Response) {
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert!(response.headers().get("set-cookie").is_none());
    assert_eq!(text_body(response).await, "");
}

#[sqlx::test(migrations = "../../migrations")]
async fn sign_up_creates_the_user_and_delivers_the_link(pool: PgPool) {
    let config = config_with(&[]);
    let mail = AsyncStubTransport::new_ok();
    let app = api_http::app(&config, services(&config, pool.clone(), mail.clone()));

    assert_accepted(post_json(app, "/users", &sign_up_body(EMAIL), fresh_client()).await).await;

    let unverified_user = "SELECT count(*) FROM users WHERE email = $1 AND NOT email_verified";
    let credential_account = "SELECT count(*) FROM accounts JOIN users ON users.id = accounts.user_id
        WHERE users.email = $1 AND accounts.provider = 'credential' AND accounts.password_hash IS NOT NULL";
    let open_verification = "SELECT count(*) FROM verifications
        WHERE identifier = $1 AND purpose = 'email_verification' AND consumed_at IS NULL";
    let dispatch =
        "SELECT count(*) FROM email_dispatches WHERE email = $1 AND kind = 'verification'";
    assert_eq!(count(&pool, unverified_user, EMAIL).await, 1);
    assert_eq!(count(&pool, credential_account, EMAIL).await, 1);
    assert_eq!(count(&pool, open_verification, EMAIL).await, 1);
    assert_eq!(count(&pool, dispatch, EMAIL).await, 1);

    assert!(eventually(|| async { !mail.messages().await.is_empty() }).await);
    let messages = mail.messages().await;
    assert_eq!(messages.len(), 1);
    let (envelope, raw) = &messages[0];
    assert_eq!(envelope.to()[0].to_string(), EMAIL);
    assert!(
        raw.contains("Subject: Confirme seu e-mail no Clinicore"),
        "{raw}"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_repeated_sign_up_writes_nothing_sends_nothing_and_answers_the_same(pool: PgPool) {
    let config = config_with(&[]);
    let setup = api_http::app(
        &config,
        services(&config, pool.clone(), AsyncStubTransport::new_ok()),
    );
    assert_accepted(post_json(setup, "/users", &sign_up_body(EMAIL), fresh_client()).await).await;

    let mail = AsyncStubTransport::new_ok();
    let app = api_http::app(&config, services(&config, pool.clone(), mail.clone()));
    let client = fresh_client();
    for _ in 0..3 {
        assert_accepted(post_json(app.clone(), "/users", &sign_up_body(EMAIL), client).await).await;
    }

    let users = "SELECT count(*) FROM users WHERE email = $1";
    let accounts = "SELECT count(*) FROM accounts JOIN users ON users.id = accounts.user_id WHERE users.email = $1";
    assert_eq!(count(&pool, users, EMAIL).await, 1);
    assert_eq!(count(&pool, accounts, EMAIL).await, 1);

    let racing = fresh_client();
    let racing_body = sign_up_body("bia@exemplo.com");
    let (first, second) = tokio::join!(
        post_json(app.clone(), "/users", &racing_body, racing),
        post_json(app.clone(), "/users", &racing_body, racing),
    );
    assert_accepted(first).await;
    assert_accepted(second).await;
    assert_eq!(count(&pool, users, "bia@exemplo.com").await, 1);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let delivered: Vec<String> = mail
        .messages()
        .await
        .iter()
        .map(|(envelope, _)| envelope.to()[0].to_string())
        .collect();
    assert!(!delivered.iter().any(|to| to == EMAIL), "{delivered:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failing_smtp_keeps_the_sign_up_and_the_answer_and_logs_the_failure(pool: PgPool) {
    let config = config_with(&[("LOG_LEVEL", Some("error"))]);
    let log = CapturedLog::default();
    let writer = log.clone();
    let _subscriber =
        tracing::subscriber::set_default(api_http::telemetry::subscriber(&config, move || {
            writer.clone()
        }));
    let app = api_http::app(
        &config,
        services(&config, pool.clone(), AsyncStubTransport::new_error()),
    );

    assert_accepted(post_json(app, "/users", &sign_up_body(EMAIL), fresh_client()).await).await;

    let users = "SELECT count(*) FROM users WHERE email = $1";
    let accounts = "SELECT count(*) FROM accounts JOIN users ON users.id = accounts.user_id WHERE users.email = $1";
    let dispatches = "SELECT count(*) FROM email_dispatches WHERE email = $1";
    assert_eq!(count(&pool, users, EMAIL).await, 1);
    assert_eq!(count(&pool, accounts, EMAIL).await, 1);
    assert_eq!(count(&pool, dispatches, EMAIL).await, 1);

    let failure_logged = || async {
        log.lines().iter().any(|line| {
            line["level"] == "ERROR" && line["email"] == EMAIL && line["reason"] == "stub error"
        })
    };
    assert!(eventually(failure_logged).await, "{:?}", log.lines());
}
