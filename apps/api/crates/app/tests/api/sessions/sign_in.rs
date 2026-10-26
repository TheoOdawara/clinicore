use std::time::Instant;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::{Session, Transport, lifetime_matches, session_exists, sign_in};
use crate::faults::faulty_app;
use crate::support::{
    MOBILE, PASSWORD, app_with, config_with, cookie_value, count, fresh_client, fresh_email,
    json_body, queued_mail, register, request, request_from, set_cookie, state,
};

#[sqlx::test(migrations = "../../migrations")]
async fn web_sign_in_opens_the_session_in_two_cookies(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let client = fresh_client();

    let body = json!({"email": email, "password": PASSWORD});
    let response = request_from(
        client,
        app.clone(),
        "POST",
        "/sessions",
        &[("user-agent", "Firefox de teste")],
        Some(&body),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()["location"], "/sessions/current");
    assert_eq!(response.headers()["cache-control"], "no-store");
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

    let session = Session {
        access: cookie_value(&access).into(),
        refresh: cookie_value(&refresh).into(),
    };
    let body = json_body(response).await;
    assert!(body["tokens"].is_null(), "{body}");
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
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
        json!({"user": {"id": user_id, "name": "Ana Souza", "email": email, "emailVerified": true, "image": null}})
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn mobile_sign_in_answers_the_tokens_and_no_cookie(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;

    let response = sign_in(&app, email, PASSWORD, &[MOBILE]).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()["location"], "/sessions/current");
    assert!(response.headers().get("set-cookie").is_none());
    let body = json_body(response).await;
    assert_eq!(body["user"]["email"], email);
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
async fn an_https_api_sets_secure_cookies(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let config = config_with(&[("API_URL", Some("https://api.clinicore.com.br"))]);
    let app = clinicore_app::app(&config, state(&config, pool.clone()));
    register(&app, &pool, email, true).await;

    let response = sign_in(&app, email, PASSWORD, &[]).await;

    for name in ["clinicore_access", "clinicore_refresh"] {
        let cookie = set_cookie(&response, name).expect("a session cookie");
        for expected in ["Secure", "HttpOnly", "SameSite=Lax"] {
            assert!(cookie.contains(expected), "{cookie}");
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_client_header_is_refused_on_any_route(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let desktop = [("clinicore-client", "desktop")];

    let response = sign_in(&app, email, PASSWORD, &desktop).await;
    let health = request(app.clone(), "GET", "/health", &desktop, None).await;

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
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;

    let mut wrong_password = Vec::new();
    let mut unknown_email = Vec::new();
    let expected = json!({"type": "tag:clinicore.com.br,2026:invalid-credentials", "title": "Invalid email or password", "status": 401});
    for _ in 0..7 {
        for (address, timings) in [
            (email.to_string(), &mut wrong_password),
            (fresh_email(), &mut unknown_email),
        ] {
            let started = Instant::now();
            let response = sign_in(&app, &address, "Errada#2026", &[]).await;
            timings.push(started.elapsed());
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(
                response.headers()["www-authenticate"],
                r#"Bearer realm="clinicore""#
            );
            assert_eq!(json_body(response).await, expected);
        }
    }
    wrong_password.sort();
    unknown_email.sort();
    let (wrong_password, unknown_email) = (wrong_password[3], unknown_email[3]);

    let gap = wrong_password.abs_diff(unknown_email);
    assert!(
        gap < wrong_password.max(unknown_email) / 2,
        "{wrong_password:?} vs {unknown_email:?}"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unverified_email_is_refused_and_gets_a_new_link(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, false).await;
    sqlx::query("DELETE FROM email_dispatches")
        .execute(&pool)
        .await
        .expect("no recent dispatch");
    let links = "SELECT count(*) FROM verifications WHERE consumed_at IS NULL";
    let before = count(&pool, links).await;

    let wrong = sign_in(&app, email, "Errada#2026", &[]).await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(count(&pool, links).await, before);

    let response = sign_in(&app, email, PASSWORD, &[]).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.headers().get("set-cookie").is_none());
    assert_eq!(
        json_body(response).await,
        json!({"type": "tag:clinicore.com.br,2026:email-not-verified", "title": "Email not verified", "status": 403})
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
    assert_eq!(count(&pool, links).await, before + 1);
    let queued = queued_mail(&pool, email).await;
    assert_eq!(queued.len(), 2);
    assert_eq!(queued[1].subject, "Confirme seu e-mail no Clinicore");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_sixth_sign_in_drops_the_oldest_session(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;

    let oldest = Transport::Web.sign_in(&app, email).await;
    for _ in 0..5 {
        Transport::Mobile.sign_in(&app, email).await;
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
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sign_in_that_waited_for_the_lock_keeps_its_own_session(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    for _ in 0..5 {
        Transport::Mobile.sign_in(&app, email).await;
    }
    sqlx::query("UPDATE sessions SET created_at = now() + interval '1 minute'")
        .execute(&pool)
        .await
        .expect("sessions that started after the next sign-in");

    let latest = Transport::Web.sign_in(&app, email).await;

    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 5);
    let current = Transport::Web
        .call(&app, "GET", "/sessions/current", &latest.access)
        .await;
    assert_eq!(current.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_sign_in_limit_refuses_the_sixth_attempt_even_with_a_forged_forwarded_for(
    pool: PgPool,
) {
    let email = fresh_email();
    let email = email.as_str();
    let config = config_with(&[("CLIENT_IP_SOURCE", Some("RightmostXForwardedFor"))]);
    let app = clinicore_app::app(&config, state(&config, pool.clone()));
    let proxy_hop = fresh_client().ip().to_string();

    for attempt in 1..=6 {
        let forwarded = format!("198.51.100.{attempt}, {proxy_hop}");
        let response = sign_in(
            &app,
            email,
            "Errada#2026",
            &[("x-forwarded-for", forwarded.as_str())],
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

#[sqlx::test(migrations = "../../migrations")]
async fn ten_failures_lock_the_account_whatever_the_address(pool: PgPool) {
    let email = fresh_email();
    let app = app_with(&pool);
    register(&app, &pool, &email, true).await;

    for attempt in 1..=10 {
        let response = sign_in(&app, &email, "Errada#2026", &[]).await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "attempt {attempt}"
        );
    }
    let response = sign_in(&app, &email, PASSWORD, &[]).await;

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        json_body(response).await,
        json!({"type": "tag:clinicore.com.br,2026:rate-limited", "title": "Too many requests", "status": 429})
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failure_at_any_step_of_the_sign_in_answers_500_or_503_and_opens_nothing(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let faulty = faulty_app(&pool).await;
    register(&faulty.app, &pool, email, true).await;

    for (fault, command, status) in [
        (
            &faulty.database,
            "JOIN accounts",
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
            "INSERT INTO sessions",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            &faulty.database,
            "DELETE FROM sessions",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            &faulty.database,
            "COMMIT",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ] {
        fault.cut_at(command, 0);
        let response = sign_in(&faulty.app, email, PASSWORD, &[]).await;
        assert_eq!(response.status(), status, "{command}");
        assert_eq!(
            count(&pool, "SELECT count(*) FROM sessions").await,
            0,
            "{command}"
        );
    }

    for _ in 0..5 {
        Transport::Mobile.sign_in(&faulty.app, email).await;
    }
    faulty.redis.cut_at("SETEX", 0);
    let evicting = sign_in(&faulty.app, email, PASSWORD, &[]).await;
    assert_eq!(evicting.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 5);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failure_sending_the_new_link_to_an_unverified_email_answers_500(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let faulty = faulty_app(&pool).await;
    register(&faulty.app, &pool, email, false).await;

    faulty.database.cut_at("pg_advisory_xact_lock", 0);
    let response = sign_in(&faulty.app, email, PASSWORD, &[]).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_request_with_no_client_address_counts_in_the_unknown_bucket(pool: PgPool) {
    let config = config_with(&[("CLIENT_IP_SOURCE", Some("RightmostXForwardedFor"))]);
    let app = clinicore_app::app(&config, state(&config, pool.clone()));
    let mut redis = redis::Client::open(
        std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run"),
    )
    .expect("a redis url")
    .get_multiplexed_async_connection()
    .await
    .expect("a redis connection");
    let unknown_buckets = || redis::cmd("KEYS").arg("rate:*:unknown").clone();
    let stale: Vec<String> = unknown_buckets()
        .query_async(&mut redis)
        .await
        .expect("the unknown buckets");
    for key in stale {
        redis::cmd("DEL")
            .arg(key)
            .query_async::<()>(&mut redis)
            .await
            .expect("a cleared bucket");
    }

    let signed_in = sign_in(&app, &fresh_email(), "Errada#2026", &[]).await;
    let confirmed = request(
        app,
        "POST",
        "/email-verifications/confirmation",
        &[MOBILE],
        Some(&json!({"token": "c".repeat(43)})),
    )
    .await;

    assert_eq!(signed_in.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(confirmed.status(), StatusCode::BAD_REQUEST);
    let buckets: Vec<String> = unknown_buckets()
        .query_async(&mut redis)
        .await
        .expect("the unknown buckets");
    assert!(buckets.len() >= 2, "{buckets:?}");
}
