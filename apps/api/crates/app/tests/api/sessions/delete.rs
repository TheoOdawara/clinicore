use axum::http::{Method, StatusCode, header};
use clinicore_app::router;
use redis::AsyncCommands;
use sqlx::PgPool;

use crate::support::{
    account, app_on, assert_problem, count, current_as_mobile, current_as_web, fresh_email,
    mobile_session, send_with, session_id_of, state_on, web_session,
};

#[sqlx::test(migrations = "../../migrations")]
async fn signing_out_drops_the_session_at_once(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let web = web_session(app_on(pool.clone()), &email).await;
    let session_id = session_id_of(&web.refresh);

    let answer = current_as_web(app_on(pool.clone()), Method::DELETE, &web.access).await;

    assert_eq!(answer.status, StatusCode::NO_CONTENT);
    assert_eq!(answer.headers[header::CACHE_CONTROL], "no-store");
    let cookies: Vec<&str> = answer
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().expect("ascii"))
        .collect();
    assert_eq!(
        cookies,
        [
            "clinicore_access=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax",
            "clinicore_refresh=; Path=/sessions/current/tokens; Max-Age=0; HttpOnly; SameSite=Lax",
        ]
    );
    let redis_url = std::env::var("REDIS_URL").expect("the tests run with REDIS_URL");
    let mut redis = clinicore_core::redis::connect(&redis_url).expect("the url parses");
    let ttl: i64 = redis
        .ttl(format!("auth:revoked:{session_id}"))
        .await
        .expect("redis answers");
    assert!((604_790..=604_800).contains(&ttl), "{ttl}");
    assert_eq!(count(&pool, "sessions").await, 0);

    let current = current_as_web(app_on(pool.clone()), Method::GET, &web.access).await;
    let again = current_as_web(app_on(pool), Method::DELETE, &web.access).await;

    assert_problem(&current, StatusCode::UNAUTHORIZED, "invalid-session");
    assert_problem(&again, StatusCode::UNAUTHORIZED, "invalid-session");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_app_signs_out_without_cookies(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let mobile = mobile_session(app_on(pool.clone()), &email).await;

    let answer = current_as_mobile(app_on(pool.clone()), Method::DELETE, &mobile.access).await;

    assert_eq!(answer.status, StatusCode::NO_CONTENT);
    assert!(!answer.headers.contains_key(header::SET_COOKIE));
    assert_eq!(count(&pool, "sessions").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn with_redis_down_signing_out_deletes_nothing(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let mobile = mobile_session(app_on(pool.clone()), &email).await;
    let app = router(state_on(
        pool.clone(),
        &[("REDIS_URL", "redis://127.0.0.1:1")],
    ));

    let answer = current_as_mobile(app.clone(), Method::DELETE, &mobile.access).await;

    assert_problem(
        &answer,
        StatusCode::SERVICE_UNAVAILABLE,
        "service-unavailable",
    );
    assert_eq!(count(&pool, "sessions").await, 1);
    assert_eq!(
        send_with(app, Method::GET, "/health", &[]).await.status,
        StatusCode::OK
    );
}
