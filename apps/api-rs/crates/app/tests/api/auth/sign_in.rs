use std::time::{Duration, Instant};

use axum::http::StatusCode;
use lettre::transport::stub::AsyncStubTransport;
use serde_json::json;
use sqlx::PgPool;

use super::{
    MOBILE, PASSWORD, Transport, app_with, count, lifetime_matches, register, session_exists,
    sign_in,
};
use crate::support::{
    config_with, eventually, fresh_client, json_body, request, set_cookie, state,
};

const EMAIL: &str = "ana@example.com";

#[sqlx::test(migrations = "../../migrations")]
async fn web_sign_in_opens_the_session_in_two_cookies(pool: PgPool) {
    let app = app_with(&pool);
    register(&app, &pool, EMAIL, true).await;
    let client = fresh_client();

    let response = sign_in(
        &app,
        EMAIL,
        PASSWORD,
        &[("user-agent", "Firefox de teste")],
        client,
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()["location"], "/sessions/current");
    let access = set_cookie(&response, "clinicore_access").expect("an access cookie");
    let refresh = set_cookie(&response, "clinicore_refresh").expect("a refresh cookie");
    for expected in ["HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=900"] {
        assert!(access.contains(expected), "{access}");
    }
    for expected in [
        "HttpOnly",
        "SameSite=Lax",
        "Path=/sessions/current/tokens",
        "Max-Age=86400",
    ] {
        assert!(refresh.contains(expected), "{refresh}");
    }
    for cookie in [&access, &refresh] {
        assert!(!cookie.contains("Secure"), "{cookie}");
        assert!(!cookie.contains("Domain"), "{cookie}");
    }

    let session = Transport::Web.session_from(response).await;
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("the user");
    let (_, secret) = session.refresh.split_once('.').expect("<session>.<secret>");
    let stored: bool = sqlx::query_scalar(
        "SELECT user_id = $2 AND client = 'web'
            AND refresh_token_hash = encode(sha256($3::bytea), 'hex')
            AND ip_address = $4::inet AND user_agent = 'Firefox de teste'
        FROM sessions WHERE id = $1",
    )
    .bind(session.id())
    .bind(user_id)
    .bind(secret)
    .bind(client.ip().to_string())
    .fetch_one(&pool)
    .await
    .expect("the session");
    assert!(stored);
    assert!(lifetime_matches(&pool, session.id(), 86400).await);

    let current = Transport::Web
        .call(&app, "GET", "/sessions/current", &session.access)
        .await;
    assert_eq!(current.status(), StatusCode::OK);
    assert_eq!(
        json_body(current).await,
        json!({"user": {"id": user_id, "name": "Ana Souza", "email": EMAIL, "emailVerified": true, "image": null}})
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn mobile_sign_in_answers_the_tokens_and_no_cookie(pool: PgPool) {
    let app = app_with(&pool);
    register(&app, &pool, EMAIL, true).await;

    let response = sign_in(&app, EMAIL, PASSWORD, &[MOBILE], fresh_client()).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()["location"], "/sessions/current");
    assert!(response.headers().get("set-cookie").is_none());
    let body = json_body(response).await;
    assert_eq!(body["user"]["email"], EMAIL);
    assert_eq!(body["tokens"]["accessTokenExpiresIn"], 900);
    let refresh = body["tokens"]["refreshToken"]
        .as_str()
        .expect("a refresh token");
    let (session_id, secret) = refresh.split_once('.').expect("<session>.<secret>");
    assert_eq!(secret.len(), 43);
    let session_id = uuid::Uuid::parse_str(session_id).expect("a session id");
    let mobile = count(
        &pool,
        "SELECT count(*) FROM sessions WHERE client = 'mobile'",
    )
    .await;
    assert_eq!(mobile, 1);
    assert!(lifetime_matches(&pool, session_id, 604800).await);

    let access = body["tokens"]["accessToken"]
        .as_str()
        .expect("an access token");
    let current = Transport::Mobile
        .call(&app, "GET", "/sessions/current", access)
        .await;
    assert_eq!(current.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn production_cookies_are_secure(pool: PgPool) {
    let config = config_with(&[("APP_ENV", Some("production"))]);
    let app = clinicore_app::app(
        &config,
        state(&config, pool.clone(), AsyncStubTransport::new_ok()),
    );
    register(&app, &pool, EMAIL, true).await;

    let response = sign_in(&app, EMAIL, PASSWORD, &[], fresh_client()).await;

    for name in ["clinicore_access", "clinicore_refresh"] {
        let cookie = set_cookie(&response, name).expect("a session cookie");
        for expected in ["Secure", "HttpOnly", "SameSite=Lax"] {
            assert!(cookie.contains(expected), "{cookie}");
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_client_header_is_refused_on_any_route(pool: PgPool) {
    let app = app_with(&pool);
    register(&app, &pool, EMAIL, true).await;
    let desktop = [("clinicore-client", "desktop")];

    let response = sign_in(&app, EMAIL, PASSWORD, &desktop, fresh_client()).await;
    let health = request(
        app.clone(),
        "GET",
        "/health",
        &desktop,
        None,
        fresh_client(),
    )
    .await;

    for response in [response, health] {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            json_body(response).await,
            json!({"type": "tag:clinicore.com.br,2026:invalid-client", "title": "Invalid client", "status": 400})
        );
    }
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_wrong_password_and_an_unknown_email_answer_the_same_in_the_same_time(pool: PgPool) {
    let app = app_with(&pool);
    register(&app, &pool, EMAIL, true).await;

    let median_of = async |email: &str| {
        let mut timings = Vec::new();
        let mut bodies = Vec::new();
        for _ in 0..20 {
            let started = Instant::now();
            let response = sign_in(&app, email, "Errada#2026", &[], fresh_client()).await;
            timings.push(started.elapsed());
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            bodies.push(json_body(response).await);
        }
        timings.sort();
        (timings[10], bodies)
    };
    let (wrong_password, wrong_password_bodies) = median_of(EMAIL).await;
    let (unknown_email, unknown_email_bodies) = median_of("ninguem@example.com").await;

    let expected = json!({"type": "tag:clinicore.com.br,2026:invalid-credentials", "title": "Invalid email or password", "status": 401});
    for body in wrong_password_bodies.iter().chain(&unknown_email_bodies) {
        assert_eq!(body, &expected);
    }
    let gap = wrong_password.abs_diff(unknown_email);
    assert!(
        gap < Duration::from_millis(50),
        "{wrong_password:?} vs {unknown_email:?}"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unverified_email_is_refused_and_gets_a_new_link(pool: PgPool) {
    let mail = AsyncStubTransport::new_ok();
    let config = config_with(&[]);
    let app = clinicore_app::app(&config, state(&config, pool.clone(), mail.clone()));
    register(&app, &pool, EMAIL, false).await;
    sqlx::query("DELETE FROM email_dispatches")
        .execute(&pool)
        .await
        .expect("no recent dispatch");
    let links = "SELECT count(*) FROM verifications WHERE consumed_at IS NULL";
    let before = count(&pool, links).await;

    let wrong = sign_in(&app, EMAIL, "Errada#2026", &[], fresh_client()).await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(count(&pool, links).await, before);

    let response = sign_in(&app, EMAIL, PASSWORD, &[], fresh_client()).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.headers().get("set-cookie").is_none());
    assert_eq!(
        json_body(response).await,
        json!({"type": "tag:clinicore.com.br,2026:email-not-verified", "title": "Email not verified", "status": 403})
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
    assert_eq!(count(&pool, links).await, before + 1);
    let resent = || async {
        mail.messages()
            .await
            .iter()
            .any(|(_, raw)| raw.contains("Subject: Confirme seu e-mail no Clinicore"))
    };
    assert!(eventually(resent).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_sixth_sign_in_drops_the_oldest_session(pool: PgPool) {
    let app = app_with(&pool);
    register(&app, &pool, EMAIL, true).await;

    let oldest = Transport::Web.sign_in(&app, EMAIL).await;
    for _ in 0..5 {
        Transport::Mobile.sign_in(&app, EMAIL).await;
    }

    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 5);
    assert!(!session_exists(&pool, oldest.id()).await);
    let current = Transport::Web
        .call(&app, "GET", "/sessions/current", &oldest.access)
        .await;
    assert_eq!(current.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        json_body(current).await["type"],
        "tag:clinicore.com.br,2026:invalid-session"
    );

    for _ in 0..14 {
        Transport::Web.sign_in(&app, EMAIL).await;
    }
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 5);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_sign_in_limit_refuses_the_sixth_attempt_even_with_a_forged_forwarded_for(
    pool: PgPool,
) {
    let config = config_with(&[("CLIENT_IP_SOURCE", Some("RightmostXForwardedFor"))]);
    let app = clinicore_app::app(
        &config,
        state(&config, pool.clone(), AsyncStubTransport::new_ok()),
    );
    let proxy_hop = fresh_client().ip().to_string();

    for attempt in 1..=6 {
        let forwarded = format!("198.51.100.{attempt}, {proxy_hop}");
        let response = sign_in(
            &app,
            EMAIL,
            "Errada#2026",
            &[("x-forwarded-for", forwarded.as_str())],
            fresh_client(),
        )
        .await;

        let expected = if attempt <= 5 {
            StatusCode::UNAUTHORIZED
        } else {
            StatusCode::TOO_MANY_REQUESTS
        };
        assert_eq!(response.status(), expected, "attempt {attempt}");
    }
}
