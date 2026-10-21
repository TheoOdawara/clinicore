use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::support::{app_with, fresh_email, post_json, queued_mail, register, text_body};

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_link_is_queued_only_for_an_unconfirmed_account(pool: PgPool) {
    let app = app_with(&pool);
    let unconfirmed = fresh_email();
    let confirmed = fresh_email();
    register(&app, &pool, &unconfirmed, false).await;
    register(&app, &pool, &confirmed, true).await;
    sqlx::query("DELETE FROM email_outbox")
        .execute(&pool)
        .await
        .expect("an empty outbox");
    sqlx::query("DELETE FROM email_dispatches")
        .execute(&pool)
        .await
        .expect("no recent dispatch");

    for email in [&unconfirmed, &confirmed] {
        let response = post_json(
            app.clone(),
            "/email-verifications",
            &json!({"email": email}),
        )
        .await;
        assert_eq!(response.status(), StatusCode::ACCEPTED, "{email}");
        assert_eq!(text_body(response).await, "");
    }

    let queued = queued_mail(&pool, &unconfirmed).await;
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].subject, "Confirme seu e-mail no Clinicore");
    assert_eq!(queued[0].status, "pending");
    assert!(queued_mail(&pool, &confirmed).await.is_empty());
}
