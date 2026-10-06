use axum::http::StatusCode;
use lettre::transport::stub::AsyncStubTransport;
use serde_json::json;
use sqlx::PgPool;

use crate::support::{
    app, config_with, count, eventually, fresh_email, json_body, post_json, register, state,
    text_body,
};

async fn request_reset(app: &axum::Router, email: &str) {
    let response = post_json(app.clone(), "/password-resets", &json!({"email": email})).await;

    assert_eq!(response.status(), StatusCode::ACCEPTED, "{email}");
    assert_eq!(text_body(response).await, "");
}

async fn dispatches_to(pool: &PgPool, email: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM email_dispatches WHERE email = $1 AND kind = 'password_reset'",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("a count")
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reset_request_answers_the_same_and_mails_only_the_account_owner(pool: PgPool) {
    let config = config_with(&[]);
    let mail = AsyncStubTransport::new_ok();
    let app = clinicore_app::app(&config, state(&config, pool.clone(), mail.clone()));
    let owner = fresh_email();
    let stranger = fresh_email();
    register(&app, &pool, &owner, true).await;
    let is_reset = |raw: &str| raw.contains("Subject: Redefinir sua senha do Clinicore");

    request_reset(&app, &owner).await;
    request_reset(&app, &stranger).await;

    let delivered = || async { mail.messages().await.iter().any(|(_, raw)| is_reset(raw)) };
    assert!(eventually(delivered).await);
    let messages = mail.messages().await;
    let resets: Vec<_> = messages.iter().filter(|(_, raw)| is_reset(raw)).collect();
    assert_eq!(resets.len(), 1);
    let (envelope, raw) = resets[0];
    assert_eq!(envelope.to()[0].to_string(), owner);
    assert!(
        raw.contains("http://localhost:3000/reset-password?token="),
        "{raw}"
    );
    let open_reset = "SELECT count(*) FROM verifications
        WHERE purpose = 'password_reset' AND consumed_at IS NULL";
    assert_eq!(count(&pool, open_reset).await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn repeated_requests_for_one_address_write_and_send_once(pool: PgPool) {
    let config = config_with(&[]);
    let mail = AsyncStubTransport::new_ok();
    let app = clinicore_app::app(&config, state(&config, pool.clone(), mail.clone()));
    let owner = fresh_email();
    let stranger = fresh_email();
    register(&app, &pool, &owner, true).await;

    for _ in 0..4 {
        request_reset(&app, &owner).await;
    }
    for _ in 0..2 {
        request_reset(&app, &stranger).await;
    }

    assert_eq!(dispatches_to(&pool, &owner).await, 1);
    assert_eq!(dispatches_to(&pool, &stranger).await, 1);
    let resets = "SELECT count(*) FROM verifications WHERE purpose = 'password_reset'";
    assert_eq!(count(&pool, resets).await, 1);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let delivered = mail
        .messages()
        .await
        .iter()
        .filter(|(_, raw)| raw.contains("Subject: Redefinir sua senha do Clinicore"))
        .count();
    assert_eq!(delivered, 1);
}

#[tokio::test]
async fn an_unknown_field_is_refused() {
    let body = json!({"email": "ana@example.com", "redirectTo": "http://evil.example/x"});

    let response = post_json(app(&config_with(&[])), "/password-resets", &body).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json_body(response).await,
        json!({
            "type": "tag:clinicore.com.br,2026:validation-failed",
            "title": "Validation failed",
            "status": 400,
            "errors": [{"pointer": "#/redirectTo", "code": "invalid"}]
        })
    );
}
