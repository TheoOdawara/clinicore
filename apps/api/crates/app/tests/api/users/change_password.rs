use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::faults::faulty_app;
use crate::sessions::{Transport, WEB_ORIGIN, revoked_ttl, session_exists, sign_in};
use crate::support::{PASSWORD, app_with, count, fresh_email, json_body, register, request};

const PATH: &str = "/users/me/password";
const NEW_PASSWORD: &str = "Outra#Senha9";

async fn change(app: &axum::Router, access: &str, current: &str, new: &str) -> Response {
    let cookie = format!("clinicore_access={access}");
    let body = json!({"currentPassword": current, "newPassword": new});
    let headers = [WEB_ORIGIN, ("cookie", cookie.as_str())];
    request(app.clone(), "PUT", PATH, &headers, Some(&body)).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn changing_the_password_keeps_this_session_and_revokes_the_others(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let kept = Transport::Web.sign_in(&app, email).await;
    let other = Transport::Mobile.sign_in(&app, email).await;

    let response = change(&app, &kept.access, PASSWORD, NEW_PASSWORD).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 1);
    assert!(session_exists(&pool, kept.id()).await);
    assert!(revoked_ttl(other.id()).await > 0);
    let on_this_device = Transport::Web
        .call(&app, "GET", "/sessions/current", &kept.access)
        .await;
    let on_the_other = Transport::Mobile
        .call(&app, "GET", "/sessions/current", &other.access)
        .await;
    assert_eq!(on_this_device.status(), StatusCode::OK);
    assert_eq!(on_the_other.status(), StatusCode::UNAUTHORIZED);

    let again = change(&app, &kept.access, PASSWORD, "Mais#Uma7").await;
    assert_eq!(again.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json_body(again).await,
        json!({"type": "tag:clinicore.com.br,2026:invalid-password", "title": "Invalid password", "status": 400})
    );
    let signed_in = sign_in(&app, email, NEW_PASSWORD, &[]).await;
    assert_eq!(signed_in.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn no_session_or_a_weak_password_changes_nothing(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let session = Transport::Web.sign_in(&app, email).await;

    let anonymous = change(&app, "not-a-token", PASSWORD, NEW_PASSWORD).await;
    let weak = change(&app, &session.access, PASSWORD, "SemEspecial1").await;

    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        json_body(anonymous).await["type"],
        "tag:clinicore.com.br,2026:invalid-session"
    );
    assert_eq!(weak.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json_body(weak).await["errors"],
        json!([{"pointer": "#/newPassword", "code": "weak_password"}])
    );
    let signed_in = sign_in(&app, email, PASSWORD, &[]).await;
    assert_eq!(signed_in.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn wrong_current_passwords_stop_at_the_ceiling_of_the_user(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let session = Transport::Web.sign_in(&app, email).await;
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&pool)
        .await
        .expect("a user");
    let url = std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run");
    let mut connection = redis::Client::open(url)
        .expect("a redis url")
        .get_multiplexed_async_connection()
        .await
        .expect("a redis connection");
    let key = format!("rate:password-change-failures:{user_id}");

    let wrong = change(&app, &session.access, "Errada#2026", NEW_PASSWORD).await;
    let failures: i64 = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut connection)
        .await
        .expect("a failure counted");
    let _: () = redis::cmd("SET")
        .arg(&key)
        .arg(10)
        .arg("EX")
        .arg(60)
        .query_async(&mut connection)
        .await
        .expect("a ceiling already reached");
    let barred = change(&app, &session.access, PASSWORD, NEW_PASSWORD).await;

    assert_eq!(wrong.status(), StatusCode::BAD_REQUEST);
    assert_eq!(failures, 1);
    assert_eq!(barred.status(), StatusCode::TOO_MANY_REQUESTS);
    let signed_in = sign_in(&app, email, PASSWORD, &[]).await;
    assert_eq!(signed_in.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failure_at_any_step_answers_500_or_503_and_keeps_the_old_password(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let faulty = faulty_app(&pool).await;
    register(&faulty.app, &pool, email, true).await;
    let kept = Transport::Web.sign_in(&faulty.app, email).await;
    Transport::Mobile.sign_in(&faulty.app, email).await;

    for (fault, command, status) in [
        (
            &faulty.database,
            "SELECT password_hash",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (&faulty.redis, "EVALSHA", StatusCode::SERVICE_UNAVAILABLE),
        (&faulty.database, "BEGIN", StatusCode::INTERNAL_SERVER_ERROR),
        (
            &faulty.database,
            "FOR UPDATE",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            &faulty.database,
            "UPDATE accounts",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            &faulty.database,
            "DELETE FROM sessions",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (&faulty.redis, "SETEX", StatusCode::SERVICE_UNAVAILABLE),
        (
            &faulty.database,
            "COMMIT",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ] {
        fault.cut_at(command, 0);
        let response = change(&faulty.app, &kept.access, PASSWORD, NEW_PASSWORD).await;
        assert_eq!(response.status(), status, "{command}");
    }

    let with_the_old = sign_in(&faulty.app, email, PASSWORD, &[]).await;
    assert_eq!(with_the_old.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_corrupt_stored_hash_answers_500_to_a_change_and_to_a_sign_in(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let session = Transport::Web.sign_in(&app, email).await;
    sqlx::query("UPDATE accounts SET password_hash = 'not-a-phc-string'")
        .execute(&pool)
        .await
        .expect("a corrupt hash");

    let changed = change(&app, &session.access, PASSWORD, NEW_PASSWORD).await;
    let signed_in = sign_in(&app, email, PASSWORD, &[]).await;

    assert_eq!(changed.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(signed_in.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_account_without_a_password_cannot_change_one(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let session = Transport::Web.sign_in(&app, email).await;
    sqlx::query("DELETE FROM accounts WHERE provider = 'credential'")
        .execute(&pool)
        .await
        .expect("an account with Google only");

    let response = change(&app, &session.access, PASSWORD, NEW_PASSWORD).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json_body(response).await["type"],
        "tag:clinicore.com.br,2026:invalid-password"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_racing_changes_with_the_same_current_password_let_only_one_win(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let session = Transport::Web.sign_in(&app, email).await;
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&pool)
        .await
        .expect("the user");
    let mut holder = pool.begin().await.expect("a transaction");
    sqlx::query("SELECT id FROM users WHERE id = $1 FOR UPDATE")
        .bind(user_id)
        .execute(&mut *holder)
        .await
        .expect("the user row held");

    let racers = ["Primeira#Senha1", "Segunda#Senha2"].map(|new_password| {
        let app = app.clone();
        let access = session.access.clone();
        tokio::spawn(async move { change(&app, &access, PASSWORD, new_password).await.status() })
    });
    let waiting = "SELECT count(*) FROM pg_stat_activity
        WHERE datname = current_database() AND wait_event_type = 'Lock'";
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while count(&pool, waiting).await < 2 {
        assert!(
            std::time::Instant::now() < deadline,
            "the changes never met at the lock"
        );
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    holder.commit().await.expect("the row released");

    let mut statuses = Vec::new();
    for racer in racers {
        statuses.push(racer.await.expect("a racer that did not panic"));
    }
    statuses.sort();
    assert_eq!(statuses, [StatusCode::NO_CONTENT, StatusCode::BAD_REQUEST]);
}
