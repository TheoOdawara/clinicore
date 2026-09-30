use std::time::Duration;

use crate::support::{
    app, capture_log, config_with, content_type, count, eventually, json_body, post_json, state,
    text_body,
};
use axum::http::StatusCode;
use lettre::transport::stub::AsyncStubTransport;
use serde_json::json;
use sqlx::PgPool;

const EMAIL: &str = "ana@exemplo.com";

fn sign_up_body(email: &str) -> serde_json::Value {
    json!({"name": "Ana", "email": email, "password": "Clinica#2026"})
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
    let app = clinicore_app::app(&config, state(&config, pool.clone(), mail.clone()));

    assert_accepted(post_json(app, "/users", &sign_up_body(EMAIL)).await).await;

    let unverified_user = "SELECT count(*) FROM users WHERE NOT email_verified";
    let credential_account =
        "SELECT count(*) FROM accounts WHERE provider = 'credential' AND password_hash IS NOT NULL";
    let open_verification = "SELECT count(*) FROM verifications
        WHERE purpose = 'email_verification' AND consumed_at IS NULL";
    let dispatch = "SELECT count(*) FROM email_dispatches WHERE kind = 'email_verification'";
    assert_eq!(count(&pool, unverified_user).await, 1);
    assert_eq!(count(&pool, credential_account).await, 1);
    assert_eq!(count(&pool, open_verification).await, 1);
    assert_eq!(count(&pool, dispatch).await, 1);

    assert!(eventually(|| async { !mail.messages().await.is_empty() }).await);
    let messages = mail.messages().await;
    assert_eq!(messages.len(), 1);
    let (envelope, raw) = &messages[0];
    assert_eq!(envelope.to()[0].to_string(), EMAIL);
    assert!(
        raw.contains("Subject: Confirme seu e-mail no Clinicore"),
        "{raw}"
    );
    assert!(raw.contains("Message-ID: <"), "{raw}");
    assert!(raw.contains("@example.com>"), "{raw}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_repeated_sign_up_writes_nothing_sends_nothing_and_answers_the_same(pool: PgPool) {
    let config = config_with(&[]);
    let setup = clinicore_app::app(
        &config,
        state(&config, pool.clone(), AsyncStubTransport::new_ok()),
    );
    assert_accepted(post_json(setup, "/users", &sign_up_body(EMAIL)).await).await;

    let mail = AsyncStubTransport::new_ok();
    let app = clinicore_app::app(&config, state(&config, pool.clone(), mail.clone()));
    for _ in 0..3 {
        assert_accepted(post_json(app.clone(), "/users", &sign_up_body(EMAIL)).await).await;
    }

    let users = "SELECT count(*) FROM users";
    assert_eq!(count(&pool, users).await, 1);
    assert_eq!(count(&pool, "SELECT count(*) FROM accounts").await, 1);

    let racing_body = sign_up_body("bia@exemplo.com");
    let (first, second) = tokio::join!(
        post_json(app.clone(), "/users", &racing_body),
        post_json(app.clone(), "/users", &racing_body),
    );
    assert_accepted(first).await;
    assert_accepted(second).await;
    assert_eq!(count(&pool, users).await, 2);

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
    let (log, _subscriber) = capture_log(&config);
    let app = clinicore_app::app(
        &config,
        state(&config, pool.clone(), AsyncStubTransport::new_error()),
    );

    assert_accepted(post_json(app, "/users", &sign_up_body(EMAIL)).await).await;

    assert_eq!(count(&pool, "SELECT count(*) FROM users").await, 1);
    assert_eq!(count(&pool, "SELECT count(*) FROM accounts").await, 1);
    assert_eq!(
        count(&pool, "SELECT count(*) FROM email_dispatches").await,
        1
    );

    let failure_logged = || async {
        log.lines().iter().any(|line| {
            line["level"] == "ERROR"
                && line["email"] == "a***@exemplo.com"
                && line["reason"] == "stub error"
        })
    };
    assert!(eventually(failure_logged).await, "{:?}", log.lines());
}

#[tokio::test]
async fn an_invalid_body_answers_400_pointing_at_each_field() {
    let cases = [
        (
            json!({"name": " ", "email": "not-an-email", "password": "weak"}),
            json!([
                {"pointer": "#/email", "code": "email"},
                {"pointer": "#/name", "code": "length"},
                {"pointer": "#/password", "code": "weak_password"}
            ]),
        ),
        (
            json!({"name": "Ana 2 golpe.com/premio", "email": EMAIL, "password": "Clinica#2026"}),
            json!([{"pointer": "#/name", "code": "name"}]),
        ),
        (
            json!({"name": 5, "email": EMAIL, "password": "Clinica#2026"}),
            json!([{"pointer": "#/name", "code": "invalid"}]),
        ),
        (
            json!({"name": "Ana", "email": EMAIL, "password": "Clinica#2026", "role": "admin"}),
            json!([{"pointer": "#/role", "code": "invalid"}]),
        ),
    ];

    for (body, errors) in cases {
        let response = post_json(app(&config_with(&[])), "/users", &body).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{body}");
        assert!(content_type(&response).starts_with("application/problem+json"));
        assert_eq!(
            json_body(response).await,
            json!({
                "type": "tag:clinicore.com.br,2026:validation-failed",
                "title": "Validation failed",
                "status": 400,
                "errors": errors
            }),
            "{body}"
        );
    }
}
