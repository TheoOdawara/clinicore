use crate::faults::faulty_app;
use crate::support::{
    MOBILE, app, app_with, config_with, content_type, count, fresh_client, json_body, post_json,
    queued_mail, request, text_body,
};
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

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
async fn sign_up_creates_the_user_and_queues_the_link_with_the_token(pool: PgPool) {
    let app = app_with(&pool);

    assert_accepted(post_json(app.clone(), "/users", &sign_up_body(EMAIL)).await).await;

    let unverified_user = "SELECT count(*) FROM users WHERE NOT email_verified";
    let credential_account =
        "SELECT count(*) FROM accounts WHERE provider = 'credential' AND password_hash IS NOT NULL";
    let open_verification = "SELECT count(*) FROM verifications
        WHERE purpose = 'email_verification' AND consumed_at IS NULL";
    let dispatch = "SELECT count(*) FROM email_dispatches WHERE kind = 'email_verification'";
    let same_expiry = "SELECT count(*) FROM email_outbox
        JOIN verifications USING (email) WHERE email_outbox.expires_at = verifications.expires_at";
    assert_eq!(count(&pool, unverified_user).await, 1);
    assert_eq!(count(&pool, credential_account).await, 1);
    assert_eq!(count(&pool, open_verification).await, 1);
    assert_eq!(count(&pool, dispatch).await, 1);
    assert_eq!(count(&pool, same_expiry).await, 1);

    let queued = queued_mail(&pool, EMAIL).await;
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].subject, "Confirme seu e-mail no Clinicore");
    assert_eq!(queued[0].status, "pending");
    assert_eq!(queued[0].attempts, 0);

    let confirmation = request(
        app,
        "POST",
        "/email-verifications/confirmation",
        &[MOBILE],
        Some(&json!({"token": queued[0].link_token()})),
    )
    .await;
    assert_eq!(confirmation.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_repeated_sign_up_writes_nothing_queues_nothing_and_answers_the_same(pool: PgPool) {
    let app = app_with(&pool);
    assert_accepted(post_json(app.clone(), "/users", &sign_up_body(EMAIL)).await).await;

    for _ in 0..3 {
        assert_accepted(post_json(app.clone(), "/users", &sign_up_body(EMAIL)).await).await;
    }

    let users = "SELECT count(*) FROM users";
    assert_eq!(count(&pool, users).await, 1);
    assert_eq!(count(&pool, "SELECT count(*) FROM accounts").await, 1);
    assert_eq!(queued_mail(&pool, EMAIL).await.len(), 1);

    let racing_body = sign_up_body("bia@exemplo.com");
    let (first, second) = tokio::join!(
        post_json(app.clone(), "/users", &racing_body),
        post_json(app.clone(), "/users", &racing_body),
    );
    assert_accepted(first).await;
    assert_accepted(second).await;
    assert_eq!(count(&pool, users).await, 2);
    assert_eq!(queued_mail(&pool, "bia@exemplo.com").await.len(), 1);
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

#[sqlx::test(migrations = "../../migrations")]
async fn a_database_failure_at_any_step_answers_500_and_keeps_nothing(pool: PgPool) {
    let faulty = faulty_app(&pool).await;

    for command in [
        "BEGIN",
        "INSERT INTO users",
        "pg_advisory_xact_lock",
        "FROM email_dispatches",
        "INSERT INTO email_dispatches",
        "INSERT INTO verifications",
        "COMMIT",
    ] {
        faulty.database.cut_at(command, 0);
        let response = post_json(faulty.app.clone(), "/users", &sign_up_body(EMAIL)).await;

        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{command}"
        );
        assert_eq!(
            count(&pool, "SELECT count(*) FROM users").await,
            0,
            "{command}"
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sign_up_right_after_a_link_request_creates_the_user_without_a_second_mail(pool: PgPool) {
    let faulty = faulty_app(&pool).await;
    let requested = post_json(
        faulty.app.clone(),
        "/email-verifications",
        &json!({"email": EMAIL}),
    )
    .await;
    assert_eq!(requested.status(), StatusCode::ACCEPTED);

    faulty.database.cut_at("COMMIT", 0);
    let failed = post_json(faulty.app.clone(), "/users", &sign_up_body(EMAIL)).await;
    assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(count(&pool, "SELECT count(*) FROM users").await, 0);

    assert_accepted(post_json(faulty.app.clone(), "/users", &sign_up_body(EMAIL)).await).await;
    assert_eq!(count(&pool, "SELECT count(*) FROM users").await, 1);
    assert!(queued_mail(&pool, EMAIL).await.is_empty());
}

#[tokio::test]
async fn a_body_that_is_not_the_expected_json_object_is_refused() {
    let oversized = format!(
        r#"{{"name": "{}", "email": "{EMAIL}", "password": "Clinica#2026"}}"#,
        "a".repeat(200 * 1024)
    );
    for (content_type, body, status) in [
        (
            "text/plain",
            "{}".to_string(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "application/json",
            r#"{"name": "Ana", "email":"#.to_string(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "application/json",
            "[]".to_string(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "application/json",
            r#""Ana""#.to_string(),
            StatusCode::BAD_REQUEST,
        ),
        ("application/json", oversized, StatusCode::PAYLOAD_TOO_LARGE),
    ] {
        let request = axum::http::Request::post("/users")
            .header("content-type", content_type)
            .extension(axum::extract::ConnectInfo(fresh_client()))
            .body(axum::body::Body::from(body))
            .expect("a valid request");

        let response = app(&config_with(&[]))
            .oneshot(request)
            .await
            .expect("an infallible router");

        assert_eq!(response.status(), status);
    }
}
