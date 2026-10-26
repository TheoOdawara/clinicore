use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::faults::faulty_app;
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

#[sqlx::test(migrations = "../../migrations")]
async fn a_database_failure_at_any_step_answers_500_and_queues_nothing(pool: PgPool) {
    let email = fresh_email();
    let faulty = faulty_app(&pool).await;
    register(&faulty.app, &pool, &email, false).await;
    sqlx::raw_sql("DELETE FROM email_outbox; DELETE FROM email_dispatches")
        .execute(&pool)
        .await
        .expect("no queued mail and no recent dispatch");

    for command in [
        "BEGIN",
        "pg_advisory_xact_lock",
        "FROM email_dispatches",
        "INSERT INTO email_dispatches",
        "AND NOT email_verified",
        "INSERT INTO verifications",
        "COMMIT",
    ] {
        faulty.database.cut_at(command, 0);
        let response = post_json(
            faulty.app.clone(),
            "/email-verifications",
            &json!({"email": email}),
        )
        .await;

        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{command}"
        );
        assert!(queued_mail(&pool, &email).await.is_empty(), "{command}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_second_request_within_the_minute_queues_nothing(pool: PgPool) {
    let email = fresh_email();
    let faulty = faulty_app(&pool).await;
    register(&faulty.app, &pool, &email, false).await;
    let body = json!({"email": email});
    let ask = || post_json(faulty.app.clone(), "/email-verifications", &body);

    faulty.database.cut_at("COMMIT", 0);
    assert_eq!(ask().await.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(ask().await.status(), StatusCode::ACCEPTED);

    assert_eq!(queued_mail(&pool, &email).await.len(), 1);
}
