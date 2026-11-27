use std::net::IpAddr;

use axum::http::{Method, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use clinicore_app::router;
use serde_json::json;
use sqlx::PgPool;

use crate::support::{
    PASSWORD, WEB_ORIGIN, account, app_on, assert_problem, cookie, count, current_as_mobile,
    current_as_web, dispatch, fresh_email, from_peer, json_request, mobile_session, random_ip,
    refresh_as_mobile, refresh_as_web, send_with, sign_in, state_on, web_session,
};

async fn lifetime_hours(pool: &PgPool, client: &str) -> f64 {
    sqlx::query_scalar(
        "SELECT extract(epoch FROM expires_at - now())::float8 / 3600 FROM sessions
         WHERE client = $1::session_client",
    )
    .bind(client)
    .fetch_one(pool)
    .await
    .expect("one session of the client exists")
}

fn claims_of(access_token: &str) -> serde_json::Value {
    let payload = access_token
        .split('.')
        .nth(1)
        .expect("the JWT has a payload");
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).expect("base64url"))
        .expect("the payload is JSON")
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_web_sign_in_opens_the_session_in_two_cookies(pool: PgPool) {
    let email = fresh_email("ana");
    let user_id = account(&pool, &email, true).await;

    let answer = sign_in(
        app_on(pool.clone()),
        &email,
        PASSWORD,
        &[("origin", WEB_ORIGIN)],
    )
    .await;

    assert_eq!(answer.status, StatusCode::CREATED);
    assert_eq!(answer.headers[header::LOCATION], "/sessions/current");
    assert_eq!(
        answer.json(),
        json!({ "user": { "id": user_id, "name": "Ana Souza", "email": email } })
    );
    let access = cookie(&answer, "clinicore_access");
    let refresh = cookie(&answer, "clinicore_refresh");
    assert!(
        access.ends_with("; Path=/; Max-Age=900; HttpOnly; SameSite=Lax"),
        "{access}"
    );
    assert!(
        refresh.ends_with("; Path=/sessions/current/tokens; Max-Age=86400; HttpOnly; SameSite=Lax"),
        "{refresh}"
    );
    assert!(!access.contains("Domain") && !refresh.contains("Domain"));
    assert_eq!(answer.headers.get_all(header::SET_COOKIE).iter().count(), 2);
    let hours = lifetime_hours(&pool, "web").await;
    assert!((23.9..=24.0).contains(&hours), "{hours}");

    let access = crate::support::cookie_value(&answer, "clinicore_access");
    let current = current_as_web(app_on(pool), Method::GET, &access).await;
    assert_eq!(current.status, StatusCode::OK);
    assert_eq!(current.json()["user"]["id"], json!(user_id));
    assert!(current.json().get("tokens").is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn in_production_both_cookies_are_secure(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let app = router(state_on(pool, &[("APP_ENV", "production")]));

    let answer = sign_in(app, &email, PASSWORD, &[("origin", WEB_ORIGIN)]).await;

    assert_eq!(answer.status, StatusCode::CREATED);
    for name in ["clinicore_access", "clinicore_refresh"] {
        assert!(cookie(&answer, name).ends_with("; SameSite=Lax; Secure"));
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_app_sign_in_returns_the_tokens_in_the_body(pool: PgPool) {
    let email = fresh_email("ana");
    let user_id = account(&pool, &email, true).await;

    let answer = sign_in(
        app_on(pool.clone()),
        &email,
        PASSWORD,
        &[("clinicore-client", "mobile")],
    )
    .await;

    assert_eq!(answer.status, StatusCode::CREATED);
    assert!(!answer.headers.contains_key(header::SET_COOKIE));
    let body = answer.json();
    assert_eq!(body["user"]["id"], json!(user_id));
    assert_eq!(body["tokens"]["accessTokenExpiresIn"], 900);
    let access = body["tokens"]["accessToken"].as_str().expect("a string");
    let claims = claims_of(access);
    assert_eq!(claims["cli"], "mobile");
    assert_eq!(claims["sub"], json!(user_id));
    assert_eq!(
        claims["exp"].as_u64().expect("a number") - claims["iat"].as_u64().expect("a number"),
        900
    );
    let refresh = body["tokens"]["refreshToken"].as_str().expect("a string");
    assert_eq!(claims["sid"].as_str(), refresh.split('.').next());
    let hours = lifetime_hours(&pool, "mobile").await;
    assert!((167.9..=168.0).contains(&hours), "{hours}");

    let current = current_as_mobile(app_on(pool), Method::GET, access).await;
    assert_eq!(current.status, StatusCode::OK);
    assert_eq!(current.json()["user"]["email"], json!(email));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_session_keeps_the_ip_and_the_user_agent_cut_at_512(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let agent = "a".repeat(600);
    let request = json_request(
        "/sessions",
        &json!({ "email": email, "password": PASSWORD }),
        &[("user-agent", agent.as_str())],
    );
    let peer: IpAddr = "203.0.113.9".parse().expect("an address");

    let answer = dispatch(app_on(pool.clone()), from_peer(request, peer)).await;

    assert_eq!(answer.status, StatusCode::CREATED);
    let (ip, stored_agent): (String, String) =
        sqlx::query_as("SELECT host(ip_address), user_agent FROM sessions")
            .fetch_one(&pool)
            .await
            .expect("the session exists");
    assert_eq!(ip, "203.0.113.9");
    assert_eq!(stored_agent, "a".repeat(512));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_ip_and_no_user_agent_are_stored_as_null(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let request = json_request(
        "/sessions",
        &json!({ "email": email, "password": PASSWORD }),
        &[],
    );

    let answer = dispatch(app_on(pool.clone()), request).await;

    assert_eq!(answer.status, StatusCode::CREATED);
    let nulls: bool =
        sqlx::query_scalar("SELECT ip_address IS NULL AND user_agent IS NULL FROM sessions")
            .fetch_one(&pool)
            .await
            .expect("the session exists");
    assert!(nulls);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_wrong_password_and_an_unknown_email_answer_the_same(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;

    let wrong = sign_in(app_on(pool.clone()), &email, "Errada#2026", &[]).await;
    let unknown = sign_in(app_on(pool.clone()), &fresh_email("sem"), PASSWORD, &[]).await;

    for answer in [&wrong, &unknown] {
        assert_problem(answer, StatusCode::UNAUTHORIZED, "invalid-credentials");
        assert_eq!(
            answer.headers[header::WWW_AUTHENTICATE],
            r#"Bearer realm="clinicore""#
        );
        assert!(!answer.headers.contains_key(header::SET_COOKIE));
    }
    assert_eq!(wrong.body, unknown.body);
    assert_eq!(
        wrong.json(),
        json!({
            "type": "tag:clinicore.com.br,2026:invalid-credentials",
            "title": "Invalid email or password",
            "status": 401,
        })
    );
    assert_eq!(count(&pool, "sessions").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unverified_email_is_refused_and_gets_one_link_a_minute(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, false).await;

    let first = sign_in(app_on(pool.clone()), &email, PASSWORD, &[]).await;

    assert_problem(&first, StatusCode::FORBIDDEN, "email-not-verified");
    assert_eq!(first.json()["title"], "Email not verified");
    assert!(!first.headers.contains_key(header::SET_COOKIE));
    assert_eq!(count(&pool, "email_outbox").await, 1);
    assert_eq!(count(&pool, "verifications").await, 1);

    let second = sign_in(app_on(pool.clone()), &email, PASSWORD, &[]).await;

    assert_problem(&second, StatusCode::FORBIDDEN, "email-not-verified");
    assert_eq!(count(&pool, "email_outbox").await, 1);
    assert_eq!(count(&pool, "sessions").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_sixth_sign_in_evicts_the_oldest_session(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let oldest = web_session(app_on(pool.clone()), &email).await;
    web_session(app_on(pool.clone()), &email).await;
    web_session(app_on(pool.clone()), &email).await;
    mobile_session(app_on(pool.clone()), &email).await;
    mobile_session(app_on(pool.clone()), &email).await;
    let oldest_id = crate::support::session_id_of(&oldest.refresh);

    let newest = mobile_session(app_on(pool.clone()), &email).await;

    assert_eq!(count(&pool, "sessions").await, 5);
    let survived: bool = sqlx::query_scalar("SELECT exists(SELECT 1 FROM sessions WHERE id = $1)")
        .bind(oldest_id)
        .fetch_one(&pool)
        .await
        .expect("the query runs");
    assert!(!survived);
    let current = current_as_web(app_on(pool.clone()), Method::GET, &oldest.access).await;
    assert_problem(&current, StatusCode::UNAUTHORIZED, "invalid-session");
    let refreshed = refresh_as_web(app_on(pool.clone()), &oldest.refresh).await;
    assert_problem(&refreshed, StatusCode::UNAUTHORIZED, "invalid-session");
    let still = refresh_as_mobile(app_on(pool), &newest.refresh).await;
    assert_eq!(still.status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_body_is_checked_field_by_field(pool: PgPool) {
    for (body, pointer, code) in [
        (
            json!({ "email": "ana", "password": PASSWORD }),
            "#/email",
            "email",
        ),
        (
            json!({ "email": "ana@exemplo.com", "password": "" }),
            "#/password",
            "length",
        ),
        (
            json!({ "email": "ana@exemplo.com", "password": "a".repeat(129) }),
            "#/password",
            "length",
        ),
        (
            json!({ "email": "ana@exemplo.com" }),
            "#/password",
            "invalid",
        ),
    ] {
        let answer = crate::support::post_json(app_on(pool.clone()), "/sessions", &body, &[]).await;

        assert_problem(&answer, StatusCode::BAD_REQUEST, "validation-failed");
        assert_eq!(
            answer.json()["errors"],
            json!([{ "pointer": pointer, "code": code }]),
            "{body}"
        );
    }
    let longest = json!({ "email": fresh_email("ana"), "password": "a".repeat(128) });
    let answer = crate::support::post_json(app_on(pool), "/sessions", &longest, &[]).await;
    assert_problem(&answer, StatusCode::UNAUTHORIZED, "invalid-credentials");
}

async fn sign_in_from(app: &axum::Router, peer: IpAddr, headers: &[(&str, &str)]) -> StatusCode {
    let body = json!({ "email": fresh_email("sem"), "password": PASSWORD });
    dispatch(
        app.clone(),
        from_peer(json_request("/sessions", &body, headers), peer),
    )
    .await
    .status
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_sixth_sign_in_from_one_ip_in_a_minute_is_refused(pool: PgPool) {
    let app = app_on(pool);
    let peer = random_ip();
    for _ in 0..5 {
        assert_eq!(
            sign_in_from(&app, peer, &[]).await,
            StatusCode::UNAUTHORIZED
        );
    }

    let body = json!({ "email": fresh_email("sem"), "password": PASSWORD });
    let refused = dispatch(
        app.clone(),
        from_peer(json_request("/sessions", &body, &[]), peer),
    )
    .await;

    assert_problem(&refused, StatusCode::TOO_MANY_REQUESTS, "rate-limited");
    let retry_after: u64 = refused.headers[header::RETRY_AFTER]
        .to_str()
        .expect("ascii")
        .parse()
        .expect("seconds");
    assert!((1..=60).contains(&retry_after));
    assert_eq!(
        sign_in_from(&app, random_ip(), &[]).await,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn ten_failures_lock_the_email_for_fifteen_minutes(pool: PgPool) {
    let ana = fresh_email("ana");
    let bia = fresh_email("bia");
    account(&pool, &ana, true).await;
    account(&pool, &bia, true).await;
    for _ in 0..9 {
        let answer = sign_in(app_on(pool.clone()), &ana, "Errada#2026", &[]).await;
        assert_eq!(answer.status, StatusCode::UNAUTHORIZED);
    }
    let right = sign_in(app_on(pool.clone()), &ana, PASSWORD, &[]).await;
    assert_eq!(right.status, StatusCode::CREATED);
    let tenth = sign_in(app_on(pool.clone()), &ana, "Errada#2026", &[]).await;
    assert_eq!(
        tenth.status,
        StatusCode::UNAUTHORIZED,
        "the right password did not count"
    );

    let locked = sign_in(app_on(pool.clone()), &ana, PASSWORD, &[]).await;

    assert_problem(&locked, StatusCode::TOO_MANY_REQUESTS, "rate-limited");
    let retry_after: u64 = locked.headers[header::RETRY_AFTER]
        .to_str()
        .expect("ascii")
        .parse()
        .expect("seconds");
    assert!((840..=900).contains(&retry_after), "{retry_after}");
    let other = sign_in(app_on(pool), &bia, PASSWORD, &[]).await;
    assert_eq!(other.status, StatusCode::CREATED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn behind_a_proxy_the_client_cannot_escape_the_sign_in_limit(pool: PgPool) {
    let forwarded_app = router(state_on(
        pool.clone(),
        &[("CLIENT_IP_SOURCE", "x-forwarded-for:1")],
    ));
    let client = random_ip().to_string();
    for index in 0..5 {
        let forwarded = format!("1.1.1.{index}, {client}");
        let status = sign_in_from(
            &forwarded_app,
            random_ip(),
            &[("x-forwarded-for", &forwarded)],
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let forwarded = format!("1.1.1.9, {client}");
    let status = sign_in_from(
        &forwarded_app,
        random_ip(),
        &[("x-forwarded-for", &forwarded)],
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);

    let cloudflare_app = router(state_on(pool, &[("CLIENT_IP_SOURCE", "cf-connecting-ip")]));
    let client = random_ip().to_string();
    for index in 0..6 {
        let forwarded = format!("1.1.1.{index}");
        let status = sign_in_from(
            &cloudflare_app,
            random_ip(),
            &[
                ("cf-connecting-ip", &client),
                ("x-forwarded-for", &forwarded),
            ],
        )
        .await;
        let expected = if index < 5 {
            StatusCode::UNAUTHORIZED
        } else {
            StatusCode::TOO_MANY_REQUESTS
        };
        assert_eq!(status, expected);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_origin_guard_lets_the_listed_web_and_the_app_through(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;

    let listed = sign_in(
        app_on(pool.clone()),
        &email,
        PASSWORD,
        &[("origin", WEB_ORIGIN)],
    )
    .await;
    let app = sign_in(
        app_on(pool.clone()),
        &email,
        PASSWORD,
        &[("clinicore-client", "mobile")],
    )
    .await;
    let lookalike = sign_in(
        app_on(pool.clone()),
        &email,
        PASSWORD,
        &[("origin", "http://localhost:3000.evil.example")],
    )
    .await;
    let cookie_only = sign_in(
        app_on(pool.clone()),
        &email,
        PASSWORD,
        &[("cookie", "clinicore_access=x")],
    )
    .await;

    assert_eq!(listed.status, StatusCode::CREATED);
    assert_eq!(app.status, StatusCode::CREATED);
    assert_problem(&lookalike, StatusCode::FORBIDDEN, "invalid-origin");
    assert_problem(&cookie_only, StatusCode::FORBIDDEN, "invalid-origin");
    assert_eq!(count(&pool, "sessions").await, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn with_redis_down_no_session_is_opened(pool: PgPool) {
    let email = fresh_email("ana");
    account(&pool, &email, true).await;
    let app = router(state_on(
        pool.clone(),
        &[("REDIS_URL", "redis://127.0.0.1:1")],
    ));

    let answer = sign_in(app.clone(), &email, PASSWORD, &[]).await;

    assert_problem(
        &answer,
        StatusCode::SERVICE_UNAVAILABLE,
        "service-unavailable",
    );
    assert_eq!(count(&pool, "sessions").await, 0);
    let health = send_with(app, Method::GET, "/health", &[]).await;
    assert_eq!(health.status, StatusCode::OK);
}
