use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::support::{app_on, post_json};

async fn sign_up(pool: &PgPool, email: &str) {
    let body = json!({ "name": "Ana Souza", "email": email, "password": "Clinica#2026" });
    let answer = post_json(app_on(pool.clone()), "/users", &body, &[]).await;
    assert_eq!(answer.status, StatusCode::ACCEPTED);
}

async fn request_link(pool: &PgPool, email: &str) {
    let answer = post_json(
        app_on(pool.clone()),
        "/email-verifications",
        &json!({ "email": email }),
        &[],
    )
    .await;
    assert_eq!(answer.status, StatusCode::ACCEPTED);
    assert!(answer.body.is_empty());
}

async fn rows_of(pool: &PgPool, email: &str) -> [i64; 3] {
    let mut rows = [0; 3];
    for (index, sql) in [
        "SELECT count(*) FROM verifications WHERE email = $1",
        "SELECT count(*) FROM email_outbox WHERE email = $1",
        "SELECT count(*) FROM email_dispatches WHERE email = $1",
    ]
    .into_iter()
    .enumerate()
    {
        rows[index] = sqlx::query_scalar(sql)
            .bind(email)
            .fetch_one(pool)
            .await
            .expect("the table is readable");
    }
    rows
}

async fn move_dispatches_back(pool: &PgPool, interval: &str) {
    sqlx::query("UPDATE email_dispatches SET created_at = now() - $1::interval")
        .bind(interval)
        .execute(pool)
        .await
        .expect("the dispatches move back");
}

async fn record_dispatches(pool: &PgPool, email: &str, how_many: i32) {
    sqlx::query(
        "INSERT INTO email_dispatches (email, kind, created_at)
         SELECT $1, 'email_verification', now() - interval '2 minutes' - step * interval '1 hour'
         FROM generate_series(0, $2 - 1) AS step",
    )
    .bind(email)
    .bind(how_many)
    .execute(pool)
    .await
    .expect("the dispatches are recorded");
}

#[sqlx::test(migrations = "../../migrations")]
async fn asking_for_a_link_answers_the_same_and_only_the_unverified_account_gets_one(pool: PgPool) {
    sign_up(&pool, "ana@exemplo.com").await;
    sign_up(&pool, "bia@exemplo.com").await;
    sqlx::query("UPDATE users SET email_verified = true WHERE email = 'bia@exemplo.com'")
        .execute(&pool)
        .await
        .expect("bia is verified");
    move_dispatches_back(&pool, "2 minutes").await;

    for email in ["Ana@Exemplo.com", "bia@exemplo.com", "sem@exemplo.com"] {
        request_link(&pool, email).await;
    }

    assert_eq!(rows_of(&pool, "ana@exemplo.com").await, [2, 2, 2]);
    assert_eq!(rows_of(&pool, "bia@exemplo.com").await, [1, 1, 2]);
    assert_eq!(rows_of(&pool, "sem@exemplo.com").await, [0, 0, 1]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_address_limit_refuses_a_second_link_in_a_minute_and_a_sixth_in_a_day(pool: PgPool) {
    sign_up(&pool, "ana@exemplo.com").await;
    record_dispatches(&pool, "sem@exemplo.com", 1).await;
    move_dispatches_back(&pool, "30 seconds").await;

    for email in ["ana@exemplo.com", "sem@exemplo.com"] {
        let before = rows_of(&pool, email).await;
        request_link(&pool, email).await;
        assert_eq!(rows_of(&pool, email).await, before, "{email} in the minute");
    }

    sqlx::query("DELETE FROM email_dispatches")
        .execute(&pool)
        .await
        .expect("the dispatches are cleared");
    for email in ["ana@exemplo.com", "sem@exemplo.com"] {
        record_dispatches(&pool, email, 5).await;
        let before = rows_of(&pool, email).await;
        request_link(&pool, email).await;
        assert_eq!(rows_of(&pool, email).await, before, "{email} in the day");
    }

    sqlx::query(
        "DELETE FROM email_dispatches WHERE email = 'ana@exemplo.com'
         AND created_at < now() - interval '4 hours'",
    )
    .execute(&pool)
    .await
    .expect("the oldest dispatch is gone");
    request_link(&pool, "ana@exemplo.com").await;
    assert_eq!(rows_of(&pool, "ana@exemplo.com").await, [2, 2, 5]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_invalid_email_is_refused(pool: PgPool) {
    let answer = post_json(
        app_on(pool.clone()),
        "/email-verifications",
        &json!({ "email": "ana" }),
        &[],
    )
    .await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        answer.json()["errors"],
        json!([{ "pointer": "#/email", "code": "email" }])
    );
    assert_eq!(rows_of(&pool, "ana").await, [0, 0, 0]);
}
