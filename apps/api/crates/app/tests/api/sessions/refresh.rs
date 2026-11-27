use axum::http::{Method, StatusCode, header};
use serde_json::json;
use sqlx::PgPool;

use crate::support::{
    WEB_ORIGIN, account, app_on, assert_problem, cookie_value, count, current_as_mobile,
    current_as_web, fresh_email, mobile_session, post_json, refresh_as_mobile, refresh_as_web,
    send_from_anywhere, session_id_of, web_session,
};

async fn hours_left(pool: &PgPool, session_id: uuid::Uuid) -> f64 {
    sqlx::query_scalar(
        "SELECT extract(epoch FROM expires_at - now())::float8 / 3600 FROM sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await
    .expect("the session exists")
}

async fn refresh_hash(pool: &PgPool, session_id: uuid::Uuid) -> Vec<u8> {
    sqlx::query_scalar("SELECT refresh_token_hash FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("the session exists")
}

async fn age(pool: &PgPool, session_id: uuid::Uuid, interval: &str) {
    sqlx::query("UPDATE sessions SET created_at = now() - $2::interval, expires_at = now() + interval '1 hour' WHERE id = $1")
        .bind(session_id)
        .bind(interval)
        .execute(pool)
        .await
        .expect("the session is aged");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_web_refresh_rotates_and_the_app_refresh_pushes_seven_days(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let web = web_session(app_on(pool.clone()), &email).await;
    let web_id = session_id_of(&web.refresh);
    age(&pool, web_id, "2 hours").await;
    let before = refresh_hash(&pool, web_id).await;

    let answer = refresh_as_web(app_on(pool.clone()), &web.refresh).await;

    assert_eq!(answer.status, StatusCode::NO_CONTENT);
    assert!(answer.body.is_empty());
    assert_eq!(answer.headers.get_all(header::SET_COOKIE).iter().count(), 2);
    let new_refresh = cookie_value(&answer, "clinicore_refresh");
    assert_ne!(new_refresh, web.refresh);
    assert_eq!(session_id_of(&new_refresh), web_id);
    assert_ne!(refresh_hash(&pool, web_id).await, before);
    let hours = hours_left(&pool, web_id).await;
    assert!((23.9..=24.0).contains(&hours), "{hours}");
    let access = cookie_value(&answer, "clinicore_access");
    let current = current_as_web(app_on(pool.clone()), Method::GET, &access).await;
    assert_eq!(current.status, StatusCode::OK);

    let mobile = mobile_session(app_on(pool.clone()), &email).await;
    let mobile_id = session_id_of(&mobile.refresh);
    age(&pool, mobile_id, "2 hours").await;

    let answer = refresh_as_mobile(app_on(pool.clone()), &mobile.refresh).await;

    assert_eq!(answer.status, StatusCode::OK);
    assert!(!answer.headers.contains_key(header::SET_COOKIE));
    let tokens = &answer.json()["tokens"];
    assert_eq!(tokens["accessTokenExpiresIn"], 900);
    assert!(answer.json().get("user").is_none());
    let hours = hours_left(&pool, mobile_id).await;
    assert!((167.9..=168.0).contains(&hours), "{hours}");
    let access = tokens["accessToken"].as_str().expect("a string");
    let current = current_as_mobile(app_on(pool), Method::GET, access).await;
    assert_eq!(current.status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn reusing_the_previous_refresh_drops_the_session(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let web = web_session(app_on(pool.clone()), &email).await;
    let rotated = refresh_as_web(app_on(pool.clone()), &web.refresh).await;
    assert_eq!(rotated.status, StatusCode::NO_CONTENT);
    let new_access = cookie_value(&rotated, "clinicore_access");

    let reused = refresh_as_web(app_on(pool.clone()), &web.refresh).await;

    assert_problem(&reused, StatusCode::UNAUTHORIZED, "session-reused");
    assert_eq!(reused.json()["title"], "Refresh token reuse detected");
    assert_eq!(
        reused.headers[header::WWW_AUTHENTICATE],
        r#"Bearer realm="clinicore""#
    );
    assert_eq!(count(&pool, "sessions").await, 0);
    let current = current_as_web(app_on(pool), Method::GET, &new_access).await;
    assert_problem(&current, StatusCode::UNAUTHORIZED, "invalid-session");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_refresh_never_passes_thirty_days(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let mobile = mobile_session(app_on(pool.clone()), &email).await;
    let session_id = session_id_of(&mobile.refresh);
    age(&pool, session_id, "29 days 23 hours").await;

    let answer = refresh_as_mobile(app_on(pool.clone()), &mobile.refresh).await;

    assert_eq!(answer.status, StatusCode::OK);
    let capped: bool = sqlx::query_scalar(
        "SELECT expires_at = created_at + interval '30 days' FROM sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .expect("the session exists");
    assert!(capped);

    let refresh = answer.json()["tokens"]["refreshToken"]
        .as_str()
        .expect("a string")
        .to_owned();
    age(&pool, session_id, "30 days 1 minute").await;

    let answer = refresh_as_mobile(app_on(pool.clone()), &refresh).await;

    assert_problem(&answer, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_eq!(count(&pool, "sessions").await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_expired_session_does_not_refresh(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let web = web_session(app_on(pool.clone()), &email).await;
    sqlx::query("UPDATE sessions SET expires_at = now() - interval '1 second'")
        .execute(&pool)
        .await
        .expect("the session expires");

    let answer = refresh_as_web(app_on(pool), &web.refresh).await;

    assert_problem(&answer, StatusCode::UNAUTHORIZED, "invalid-session");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_refresh_token_only_works_on_the_transport_it_was_born_in(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let web = web_session(app_on(pool.clone()), &email).await;
    let mobile = mobile_session(app_on(pool.clone()), &email).await;

    let mobile_by_web = refresh_as_web(app_on(pool.clone()), &mobile.refresh).await;
    let web_by_mobile = refresh_as_mobile(app_on(pool.clone()), &web.refresh).await;

    assert_problem(&mobile_by_web, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_problem(&web_by_mobile, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_eq!(count(&pool, "sessions").await, 2);
    assert_eq!(
        refresh_as_web(app_on(pool.clone()), &web.refresh)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        refresh_as_mobile(app_on(pool), &mobile.refresh)
            .await
            .status,
        StatusCode::OK
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_missing_or_malformed_refresh_token_is_refused(pool: PgPool) {
    let no_cookie = send_from_anywhere(
        app_on(pool.clone()),
        Method::POST,
        "/sessions/current/tokens",
        &[("origin", WEB_ORIGIN)],
    )
    .await;
    let bad_cookie = refresh_as_web(app_on(pool.clone()), "not-a-token").await;
    let unknown = refresh_as_web(
        app_on(pool.clone()),
        &format!(
            "{}.{}",
            uuid::Uuid::from_u128(rand::random()),
            "a".repeat(43)
        ),
    )
    .await;
    let bad_body = post_json(
        app_on(pool),
        "/sessions/current/tokens",
        &json!({ "refreshToken": "not-a-token" }),
        &[("clinicore-client", "mobile")],
    )
    .await;

    assert_problem(&no_cookie, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_problem(&bad_cookie, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_problem(&unknown, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_problem(&bad_body, StatusCode::BAD_REQUEST, "validation-failed");
    assert_eq!(
        bad_body.json()["errors"],
        json!([{ "pointer": "#/refreshToken", "code": "invalid" }])
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_31st_refresh_of_one_session_in_a_minute_is_refused(pool: PgPool) {
    let app = app_on(pool);
    let token = format!(
        "{}.{}",
        uuid::Uuid::from_u128(rand::random()),
        "a".repeat(43)
    );
    for _ in 0..30 {
        let answer = refresh_as_mobile(app.clone(), &token).await;
        assert_problem(&answer, StatusCode::UNAUTHORIZED, "invalid-session");
    }

    let refused = refresh_as_mobile(app, &token).await;

    assert_problem(&refused, StatusCode::TOO_MANY_REQUESTS, "rate-limited");
}
