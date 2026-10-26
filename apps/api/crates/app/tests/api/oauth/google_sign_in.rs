use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::{IdToken, OAUTH_COOKIES, callback, fake_google, sign_in_with_google, start};
use crate::faults::faulty_app_with_google;
use crate::sessions::sign_in;
use crate::support::{
    PASSWORD, app_with, app_with_google, count, fresh_email, register, request, send, set_cookie,
};

const ROWS: &str = "SELECT (SELECT count(*) FROM users) + (SELECT count(*) FROM accounts)
    + (SELECT count(*) FROM sessions) + (SELECT count(*) FROM verifications)
    + (SELECT count(*) FROM email_dispatches)";
const USERS: &str = "SELECT count(*) FROM users";
const GOOGLE_ACCOUNTS: &str =
    "SELECT count(*) FROM accounts WHERE provider = 'google' AND password_hash IS NULL";
const CREDENTIAL_ACCOUNTS: &str = "SELECT count(*) FROM accounts WHERE provider = 'credential'";

fn profile(email: &str, email_verified: bool) -> serde_json::Value {
    json!({
        "sub": "google-ana",
        "email": email,
        "email_verified": email_verified,
        "name": "Ana Souza",
        "picture": "https://lh3.googleusercontent.com/a/ana",
    })
}

#[sqlx::test(migrations = "../../migrations")]
async fn starting_the_google_sign_in_writes_nothing(pool: PgPool) {
    let app = app_with(&pool);

    for _ in 0..3 {
        let response = send(app.clone(), "GET", "/oauth/google").await;

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let location = response.headers()["location"].to_str().expect("a location");
        assert!(
            location.starts_with("https://accounts.google.com/"),
            "{location}"
        );
        for expected in [
            "prompt=select_account",
            "scope=openid+email+profile",
            "code_challenge_method=S256",
            "code_challenge=",
            "nonce=",
            "redirect_uri=http%3A%2F%2Flocalhost%3A3333%2Foauth%2Fgoogle%2Fcallback",
        ] {
            assert!(location.contains(expected), "{location}");
        }
        for name in OAUTH_COOKIES {
            let cookie = set_cookie(&response, name).unwrap_or_else(|| panic!("a {name} cookie"));
            for expected in [
                "HttpOnly",
                "SameSite=Lax",
                "Path=/oauth/google",
                "Max-Age=600",
            ] {
                assert!(cookie.contains(expected), "{cookie}");
            }
            assert!(!cookie.contains("Secure"), "{cookie}");
        }
    }

    assert_eq!(count(&pool, ROWS).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn google_links_to_the_existing_account(pool: PgPool) {
    let email = fresh_email();
    let google = fake_google(IdToken::Genuine, profile(&email, true)).await;
    let app = app_with_google(&pool, google.endpoints.clone());
    register(&app, &pool, &email, true).await;

    let response = sign_in_with_google(&app, &google).await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()["location"], "http://localhost:3000/app");
    assert!(set_cookie(&response, "clinicore_access").is_some());
    assert!(set_cookie(&response, "clinicore_refresh").is_some());
    for name in OAUTH_COOKIES {
        let cleared = set_cookie(&response, name).unwrap_or_else(|| panic!("a {name} cookie"));
        assert!(cleared.contains("Max-Age=0"), "{cleared}");
    }
    assert_eq!(count(&pool, USERS).await, 1);
    assert_eq!(count(&pool, GOOGLE_ACCOUNTS).await, 1);
    let subject: String = sqlx::query_scalar(
        "SELECT accounts.provider_account_id FROM accounts
        JOIN users ON users.id = accounts.user_id
        WHERE accounts.provider = 'google' AND users.email = $1",
    )
    .bind(&email)
    .fetch_one(&pool)
    .await
    .expect("a google account of the same user");
    assert_eq!(subject, "google-ana");
    let with_password = sign_in(&app, &email, PASSWORD, &[]).await;
    assert_eq!(with_password.status(), StatusCode::CREATED);

    let again = sign_in_with_google(&app, &google).await;
    assert_eq!(again.headers()["location"], "http://localhost:3000/app");
    assert_eq!(count(&pool, GOOGLE_ACCOUNTS).await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_tampered_state_an_unverified_email_and_an_invalid_id_token_are_refused(pool: PgPool) {
    let email = fresh_email();
    let google = fake_google(IdToken::Genuine, profile(&email, false)).await;
    let app = app_with_google(&pool, google.endpoints.clone());
    let invalid_state = "http://localhost:3000/login?error=INVALID_STATE";

    let started = start(&app, &google).await;
    let tampered = callback(&app, "another-state", Some(&started.cookies)).await;
    assert_eq!(tampered.status(), StatusCode::FOUND);
    assert_eq!(tampered.headers()["location"], invalid_state);

    let without_cookie = callback(&app, &started.state, None).await;
    assert_eq!(without_cookie.headers()["location"], invalid_state);

    let unverified = callback(&app, &started.state, Some(&started.cookies)).await;
    assert_eq!(unverified.status(), StatusCode::FOUND);
    assert_eq!(
        unverified.headers()["location"],
        "http://localhost:3000/login?error=UNVERIFIED_PROVIDER_EMAIL"
    );
    assert!(set_cookie(&unverified, "clinicore_access").is_none());

    for id_token in [
        IdToken::ForAnotherClient,
        IdToken::WithAnotherNonce,
        IdToken::Forged,
    ] {
        let google = fake_google(id_token, profile(&email, true)).await;
        let app = app_with_google(&pool, google.endpoints.clone());
        let refused = sign_in_with_google(&app, &google).await;
        assert_eq!(
            refused.headers()["location"],
            "http://localhost:3000/login?error=PROVIDER_ERROR",
            "{id_token:?}"
        );
    }

    assert_eq!(count(&pool, ROWS).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn google_creates_the_account_of_a_new_email(pool: PgPool) {
    let email = fresh_email();
    let google = fake_google(IdToken::Genuine, profile(&email.to_uppercase(), true)).await;
    let app = app_with_google(&pool, google.endpoints.clone());

    let response = sign_in_with_google(&app, &google).await;

    assert_eq!(response.headers()["location"], "http://localhost:3000/app");
    let created: (String, bool, Option<String>) =
        sqlx::query_as("SELECT name, email_verified, image FROM users WHERE email = $1")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .expect("the created user");
    assert_eq!(
        created,
        (
            "Ana Souza".to_string(),
            true,
            Some("https://lh3.googleusercontent.com/a/ana".to_string())
        )
    );
    assert_eq!(count(&pool, GOOGLE_ACCOUNTS).await, 1);
    assert_eq!(count(&pool, CREDENTIAL_ACCOUNTS).await, 0);
    assert_eq!(
        count(&pool, "SELECT count(*) FROM sessions WHERE client = 'web'").await,
        1
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn linking_google_to_an_unverified_account_drops_its_password(pool: PgPool) {
    let email = fresh_email();
    let mut owner_profile = profile(&email, true);
    owner_profile["name"] = json!("Maria Lima");
    let google = fake_google(IdToken::Genuine, owner_profile).await;
    let app = app_with_google(&pool, google.endpoints.clone());
    register(&app, &pool, &email, false).await;

    let response = sign_in_with_google(&app, &google).await;

    assert_eq!(response.headers()["location"], "http://localhost:3000/app");
    assert_eq!(count(&pool, USERS).await, 1);
    assert_eq!(count(&pool, CREDENTIAL_ACCOUNTS).await, 0);
    let owner: (String, bool, Option<String>) =
        sqlx::query_as("SELECT name, email_verified, image FROM users WHERE email = $1")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .expect("the linked user");
    assert_eq!(
        owner,
        (
            "Maria Lima".to_string(),
            true,
            Some("https://lh3.googleusercontent.com/a/ana".to_string())
        )
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM verifications WHERE consumed_at IS NULL"
        )
        .await,
        0
    );
    let with_the_first_password = sign_in(&app, &email, PASSWORD, &[]).await;
    assert_eq!(with_the_first_password.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 1);
}

const PROVIDER_ERROR: &str = "http://localhost:3000/login?error=PROVIDER_ERROR";

#[sqlx::test(migrations = "../../migrations")]
async fn a_refused_code_no_id_token_unreachable_keys_or_no_email_are_provider_errors(pool: PgPool) {
    let email = fresh_email();
    let mut without_email = profile(&email, true);
    without_email
        .as_object_mut()
        .expect("a profile object")
        .remove("email");

    for (id_token, claims) in [
        (IdToken::Refused, profile(&email, true)),
        (IdToken::Absent, profile(&email, true)),
        (IdToken::KeysDown, profile(&email, true)),
        (IdToken::Genuine, without_email),
    ] {
        let google = fake_google(id_token, claims).await;
        let app = app_with_google(&pool, google.endpoints.clone());

        let refused = sign_in_with_google(&app, &google).await;

        assert_eq!(
            refused.headers()["location"],
            PROVIDER_ERROR,
            "{id_token:?}"
        );
    }
    assert_eq!(count(&pool, ROWS).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_denied_authorization_is_a_provider_error(pool: PgPool) {
    let google = fake_google(IdToken::Genuine, profile(&fresh_email(), true)).await;
    let app = app_with_google(&pool, google.endpoints.clone());
    let started = start(&app, &google).await;

    let path = format!(
        "/oauth/google/callback?error=access_denied&state={}",
        started.state
    );
    let denied = request(app, "GET", &path, &[("cookie", &started.cookies)], None).await;

    assert_eq!(denied.headers()["location"], PROVIDER_ERROR);
    assert_eq!(count(&pool, ROWS).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_account_linked_to_another_google_subject_is_a_provider_error(pool: PgPool) {
    let email = fresh_email();
    let google = fake_google(IdToken::Genuine, profile(&email, true)).await;
    let app = app_with_google(&pool, google.endpoints.clone());
    let linked = sign_in_with_google(&app, &google).await;
    assert_eq!(linked.headers()["location"], "http://localhost:3000/app");

    let mut another_subject = profile(&email, true);
    another_subject["sub"] = json!("google-another-ana");
    let another = fake_google(IdToken::Genuine, another_subject).await;
    let app = app_with_google(&pool, another.endpoints.clone());
    let refused = sign_in_with_google(&app, &another).await;

    assert_eq!(refused.headers()["location"], PROVIDER_ERROR);
    assert_eq!(count(&pool, GOOGLE_ACCOUNTS).await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failure_while_linking_or_opening_the_session_answers_500_or_503(pool: PgPool) {
    let email = fresh_email();
    let google = fake_google(IdToken::Genuine, profile(&email, true)).await;
    let faulty = faulty_app_with_google(&pool, google.endpoints.clone()).await;
    register(&faulty.app, &pool, &email, false).await;

    for route_limit_then_network_limit in [0, 1] {
        faulty.redis.cut_at("INCR", route_limit_then_network_limit);
        let starting = send(faulty.app.clone(), "GET", "/oauth/google").await;
        assert_eq!(starting.status(), StatusCode::SERVICE_UNAVAILABLE);

        let started = start(&faulty.app, &google).await;
        faulty.redis.cut_at("INCR", route_limit_then_network_limit);
        let finishing = callback(&faulty.app, &started.state, Some(&started.cookies)).await;
        assert_eq!(finishing.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    for (command, skip) in [
        ("DELETE FROM accounts", 0),
        ("UPDATE verifications", 0),
        ("UPDATE users SET email_verified", 0),
        ("SELECT user_id FROM accounts", 0),
        ("BEGIN", 0),
        ("INSERT INTO users", 0),
        ("INSERT INTO accounts", 0),
        ("SELECT exists(", 0),
        ("COMMIT", 0),
        ("BEGIN", 1),
    ] {
        let started = start(&faulty.app, &google).await;
        faulty.database.cut_at(command, skip);
        let response = callback(&faulty.app, &started.state, Some(&started.cookies)).await;
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{command}"
        );
    }

    let response = sign_in_with_google(&faulty.app, &google).await;
    assert_eq!(response.headers()["location"], "http://localhost:3000/app");
}
