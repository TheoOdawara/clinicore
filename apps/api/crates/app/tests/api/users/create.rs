use axum::http::{StatusCode, header};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::support::{
    WEB_ORIGIN, app_on, count, counts, dispatch, from_peer, post_json, random_ip, state_on,
    token_in,
};

fn ana() -> Value {
    json!({ "name": "Ana Souza", "email": "Ana@Exemplo.com", "password": "Clinica#2026" })
}

#[sqlx::test(migrations = "../../migrations")]
async fn sign_up_creates_the_account_without_a_session_and_queues_the_link(pool: PgPool) {
    let answer = post_json(app_on(pool.clone()), "/users", &ana(), &[]).await;

    assert_eq!(answer.status, StatusCode::ACCEPTED);
    assert!(answer.body.is_empty());
    assert!(!answer.headers.contains_key(header::SET_COOKIE));
    assert_eq!(answer.headers[header::CACHE_CONTROL], "no-store");

    let (name, verified): (String, bool) =
        sqlx::query_as("SELECT name, email_verified FROM users WHERE email = 'ana@exemplo.com'")
            .fetch_one(&pool)
            .await
            .expect("the user exists in lowercase");
    assert_eq!(name, "Ana Souza");
    assert!(!verified);

    let password_hash: String = sqlx::query_scalar(
        "SELECT password_hash FROM accounts JOIN users ON users.id = accounts.user_id
         WHERE users.email = 'ana@exemplo.com' AND provider = 'credential'",
    )
    .fetch_one(&pool)
    .await
    .expect("the credential account exists");
    assert!(password_hash.starts_with("$argon2id$"));

    let (token_hash, lasts_an_hour, verification_expires_at): (Vec<u8>, bool, String) =
        sqlx::query_as(
            "SELECT token_hash, expires_at - created_at = interval '1 hour', expires_at::text
             FROM verifications WHERE email = 'ana@exemplo.com' AND purpose = 'email_verification'",
        )
        .fetch_one(&pool)
        .await
        .expect("the verification exists");
    assert!(lasts_an_hour);

    let (status, subject, text, outbox_expires_at): (String, String, String, String) =
        sqlx::query_as(
            "SELECT status::text, subject, text_body, expires_at::text
             FROM email_outbox WHERE email = 'ana@exemplo.com'",
        )
        .fetch_one(&pool)
        .await
        .expect("the email is queued");
    assert_eq!(status, "pending");
    assert_eq!(subject, "Confirme seu e-mail no Clinicore");
    assert!(text.starts_with("Olá, Ana."), "{text}");
    assert_eq!(outbox_expires_at, verification_expires_at);

    let token = token_in(&text);
    assert_eq!(token.len(), 43);
    assert_eq!(token_hash, Sha256::digest(token.as_bytes()).to_vec());
}

#[sqlx::test(migrations = "../../migrations")]
async fn signing_up_an_email_that_has_an_account_writes_and_sends_nothing(pool: PgPool) {
    post_json(app_on(pool.clone()), "/users", &ana(), &[]).await;
    let before = counts(&pool).await;

    let mut again = ana();
    again["password"] = json!("Outra#Senha9");
    let answer = post_json(app_on(pool.clone()), "/users", &again, &[]).await;

    assert_eq!(answer.status, StatusCode::ACCEPTED);
    assert!(answer.body.is_empty());
    assert_eq!(counts(&pool).await, before);
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_simultaneous_sign_ups_of_one_email_create_one_account(pool: PgPool) {
    let body = ana();
    let (first, second) = tokio::join!(
        post_json(app_on(pool.clone()), "/users", &body, &[]),
        post_json(app_on(pool.clone()), "/users", &body, &[]),
    );

    assert_eq!(first.status, StatusCode::ACCEPTED);
    assert_eq!(second.status, StatusCode::ACCEPTED);
    assert!(first.body.is_empty() && second.body.is_empty());
    assert_eq!(count(&pool, "users").await, 1);
    assert_eq!(count(&pool, "accounts").await, 1);
    assert_eq!(count(&pool, "email_outbox").await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_invalid_body_is_refused_field_by_field_and_nothing_is_written(pool: PgPool) {
    let answer = post_json(
        app_on(pool.clone()),
        "/users",
        &json!({ "name": "", "email": "ana", "password": "fraca" }),
        &[],
    )
    .await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        answer.headers[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(answer.headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        answer.json(),
        json!({
            "type": "tag:clinicore.com.br,2026:validation-failed",
            "title": "Validation failed",
            "status": 400,
            "errors": [
                { "pointer": "#/name", "code": "length" },
                { "pointer": "#/email", "code": "email" },
                { "pointer": "#/password", "code": "weak_password" },
            ],
        })
    );
    assert_eq!(counts(&pool).await, [0; 5]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_missing_or_mistyped_field_is_invalid(pool: PgPool) {
    let mut extra = ana();
    extra["role"] = json!("admin");
    let answer = post_json(app_on(pool.clone()), "/users", &extra, &[]).await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        answer.json()["errors"],
        json!([{ "pointer": "#/role", "code": "invalid" }])
    );

    let answer = post_json(
        app_on(pool.clone()),
        "/users",
        &json!({ "name": "Ana Souza", "password": 2026 }),
        &[],
    )
    .await;

    assert_eq!(
        answer.json()["errors"],
        json!([
            { "pointer": "#/email", "code": "invalid" },
            { "pointer": "#/password", "code": "invalid" },
        ])
    );
    assert_eq!(counts(&pool).await, [0; 5]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_body_that_is_not_a_json_object_is_a_plain_bad_request(pool: PgPool) {
    let answer = post_json(app_on(pool.clone()), "/users", &json!([ana()]), &[]).await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
    assert_eq!(answer.json()["type"], "about:blank");
    assert_eq!(answer.json()["title"], "Bad Request");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_origin_guard_lets_the_listed_web_through_and_refuses_the_rest(pool: PgPool) {
    let app = app_on(pool.clone());

    let listed = post_json(app.clone(), "/users", &ana(), &[("origin", WEB_ORIGIN)]).await;
    let suffixed = post_json(
        app.clone(),
        "/users",
        &json!({ "name": "Bia", "email": "bia@exemplo.com", "password": "Clinica#2026" }),
        &[("origin", "http://localhost:3000.evil.example")],
    )
    .await;
    let cookie_only = post_json(
        app,
        "/users",
        &json!({ "name": "Bia", "email": "bia@exemplo.com", "password": "Clinica#2026" }),
        &[("cookie", "clinicore_access=x")],
    )
    .await;

    assert_eq!(listed.status, StatusCode::ACCEPTED);
    for refused in [&suffixed, &cookie_only] {
        assert_eq!(refused.status, StatusCode::FORBIDDEN);
        assert_eq!(
            refused.json()["type"],
            "tag:clinicore.com.br,2026:invalid-origin"
        );
    }
    assert_eq!(count(&pool, "users").await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unreachable_redis_closes_the_door_without_writing(pool: PgPool) {
    let app = clinicore_app::router(state_on(
        pool.clone(),
        &[("REDIS_URL", "redis://127.0.0.1:1")],
    ));

    let answer = post_json(app, "/users", &ana(), &[]).await;

    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        answer.json(),
        json!({
            "type": "tag:clinicore.com.br,2026:service-unavailable",
            "title": "Service temporarily unavailable",
            "status": 503,
        })
    );
    assert_eq!(answer.headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(count(&pool, "users").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_body_not_declared_as_json_or_too_large_gets_the_plain_status(pool: PgPool) {
    let undeclared = axum::http::Request::post("/users")
        .body(axum::body::Body::from(ana().to_string()))
        .expect("the request is well formed");
    let answer = dispatch(app_on(pool.clone()), from_peer(undeclared, random_ip())).await;

    assert_eq!(answer.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(answer.json()["type"], "about:blank");

    let mut huge = ana();
    huge["name"] = json!("a".repeat(3 * 1024 * 1024));
    let answer = post_json(app_on(pool.clone()), "/users", &huge, &[]).await;

    assert_eq!(answer.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(answer.json()["type"], "about:blank");
    assert_eq!(count(&pool, "users").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_database_failure_answers_a_bare_500(pool: PgPool) {
    let app = app_on(pool.clone());
    pool.close().await;

    let answer = post_json(app, "/users", &ana(), &[]).await;

    assert_eq!(answer.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        answer.json(),
        json!({ "type": "about:blank", "title": "Internal Server Error", "status": 500 })
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn sign_up_inside_the_dispatch_spacing_creates_the_account_without_the_email(pool: PgPool) {
    sqlx::query(
        "INSERT INTO email_dispatches (email, kind, created_at)
         VALUES ('ana@exemplo.com', 'email_verification', now() - interval '30 seconds')",
    )
    .execute(&pool)
    .await
    .expect("the dispatch is recorded");

    let answer = post_json(app_on(pool.clone()), "/users", &ana(), &[]).await;

    assert_eq!(answer.status, StatusCode::ACCEPTED);
    assert_eq!(counts(&pool).await, [1, 1, 0, 1, 0]);
}
