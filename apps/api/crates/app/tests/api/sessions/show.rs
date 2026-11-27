use axum::http::{Method, StatusCode, header};
use clinicore_app::http::client::Client;
use clinicore_app::sessions::SessionId;
use clinicore_app::sessions::tokens::access;
use clinicore_app::users::UserId;
use sqlx::PgPool;

use crate::support::{
    account, app_on, assert_problem, current_as_mobile, current_as_web, fresh_email,
    mobile_session, send_from_anywhere, session_id_of, web_session,
};

const SECRET: &str = "a-test-secret-with-at-least-32-characters";

#[sqlx::test(migrations = "../../migrations")]
async fn a_transport_does_not_authenticate_through_the_other(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let web = web_session(app_on(pool.clone()), &email).await;
    let mobile = mobile_session(app_on(pool.clone()), &email).await;

    let web_as_mobile = current_as_mobile(app_on(pool.clone()), Method::GET, &web.access).await;
    let mobile_as_web = current_as_web(app_on(pool.clone()), Method::GET, &mobile.access).await;
    let desktop = send_from_anywhere(
        app_on(pool.clone()),
        Method::GET,
        "/sessions/current",
        &[("clinicore-client", "desktop")],
    )
    .await;

    assert_problem(&web_as_mobile, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_problem(&mobile_as_web, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_problem(&desktop, StatusCode::BAD_REQUEST, "invalid-client");
    assert_eq!(crate::support::count(&pool, "sessions").await, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_missing_malformed_forged_or_expired_token_is_an_invalid_session(pool: PgPool) {
    let email = fresh_email("ana");
    let user_id = account(&pool, &email, true).await;
    let mobile = mobile_session(app_on(pool.clone()), &email).await;
    let session_id = session_id_of(&mobile.refresh);
    let expired = access::issue(
        SECRET,
        UserId(user_id),
        SessionId(session_id),
        Client::Mobile,
        access::now() - 900,
    );
    let forged = access::issue(
        "another-secret-with-at-least-32-characters",
        UserId(user_id),
        SessionId(session_id),
        Client::Mobile,
        access::now(),
    );

    let missing_cookie =
        send_from_anywhere(app_on(pool.clone()), Method::GET, "/sessions/current", &[]).await;
    let missing_bearer = send_from_anywhere(
        app_on(pool.clone()),
        Method::GET,
        "/sessions/current",
        &[
            ("clinicore-client", "mobile"),
            ("authorization", "Basic abc"),
        ],
    )
    .await;
    assert_problem(&missing_cookie, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_eq!(
        missing_cookie.headers[header::WWW_AUTHENTICATE],
        r#"Bearer realm="clinicore""#
    );
    assert_eq!(missing_cookie.json()["title"], "Invalid session");
    assert_problem(&missing_bearer, StatusCode::UNAUTHORIZED, "invalid-session");
    for token in ["not-a-token", expired.as_str(), forged.as_str()] {
        let answer = current_as_mobile(app_on(pool.clone()), Method::GET, token).await;
        assert_problem(&answer, StatusCode::UNAUTHORIZED, "invalid-session");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_token_whose_user_is_gone_is_an_invalid_session(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let mobile = mobile_session(app_on(pool.clone()), &email).await;
    sqlx::query("DELETE FROM users")
        .execute(&pool)
        .await
        .expect("the user is deleted");

    let answer = current_as_mobile(app_on(pool), Method::GET, &mobile.access).await;

    assert_problem(&answer, StatusCode::UNAUTHORIZED, "invalid-session");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_101st_read_of_one_session_in_ten_seconds_is_refused(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let mobile = mobile_session(app_on(pool.clone()), &email).await;
    let app = app_on(pool);
    for _ in 0..100 {
        let answer = current_as_mobile(app.clone(), Method::GET, &mobile.access).await;
        assert_eq!(answer.status, StatusCode::OK);
        assert_eq!(answer.headers[header::CACHE_CONTROL], "no-store");
    }

    let refused = current_as_mobile(app, Method::GET, &mobile.access).await;

    assert_problem(&refused, StatusCode::TOO_MANY_REQUESTS, "rate-limited");
    assert_eq!(refused.headers[header::CACHE_CONTROL], "no-store");
}
