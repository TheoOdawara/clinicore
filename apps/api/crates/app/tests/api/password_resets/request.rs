use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::support::{
    app, app_with, config_with, count, fresh_email, json_body, post_json, queued_mail, register,
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
async fn a_reset_request_answers_the_same_and_queues_mail_only_for_the_account_owner(pool: PgPool) {
    let app = app_with(&pool);
    let owner = fresh_email();
    let stranger = fresh_email();
    register(&app, &pool, &owner, true).await;

    request_reset(&app, &owner).await;
    request_reset(&app, &stranger).await;

    let resets: Vec<_> = queued_mail(&pool, &owner)
        .await
        .into_iter()
        .filter(|mail| mail.subject == "Redefinir sua senha do Clinicore")
        .collect();
    assert_eq!(resets.len(), 1);
    assert_eq!(resets[0].status, "pending");
    assert!(
        resets[0]
            .text_body
            .contains("http://localhost:3000/reset-password?token="),
        "{}",
        resets[0].text_body
    );
    assert!(queued_mail(&pool, &stranger).await.is_empty());
    let open_reset = "SELECT count(*) FROM verifications
        WHERE purpose = 'password_reset' AND consumed_at IS NULL";
    assert_eq!(count(&pool, open_reset).await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn repeated_requests_for_one_address_write_and_queue_once(pool: PgPool) {
    let app = app_with(&pool);
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
    let queued_resets = "SELECT count(*) FROM email_outbox
        WHERE subject = 'Redefinir sua senha do Clinicore'";
    assert_eq!(count(&pool, queued_resets).await, 1);
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
