use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::{callback, fake_google, sign_in_with_google, start};
use crate::sessions::sign_in;
use crate::support::{
    PASSWORD, app_with, app_with_google, count, fresh_email, register, send, set_cookie,
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
            "redirect_uri=http%3A%2F%2Flocalhost%3A3333%2Foauth%2Fgoogle%2Fcallback",
        ] {
            assert!(location.contains(expected), "{location}");
        }
        for name in ["clinicore_oauth_state", "clinicore_oauth_verifier"] {
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
    let app = app_with_google(&pool, fake_google(profile(&email, true)).await);
    register(&app, &pool, &email, true).await;

    let response = sign_in_with_google(&app).await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()["location"], "http://localhost:3000/app");
    assert!(set_cookie(&response, "clinicore_access").is_some());
    assert!(set_cookie(&response, "clinicore_refresh").is_some());
    for name in ["clinicore_oauth_state", "clinicore_oauth_verifier"] {
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

    let again = sign_in_with_google(&app).await;
    assert_eq!(again.headers()["location"], "http://localhost:3000/app");
    assert_eq!(count(&pool, GOOGLE_ACCOUNTS).await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_tampered_state_and_an_unverified_google_email_are_refused(pool: PgPool) {
    let email = fresh_email();
    let app = app_with_google(&pool, fake_google(profile(&email, false)).await);
    let invalid_state = "http://localhost:3000/login?error=INVALID_STATE";

    let started = start(&app).await;
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

    assert_eq!(count(&pool, ROWS).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn google_creates_the_account_of_a_new_email(pool: PgPool) {
    let email = fresh_email();
    let app = app_with_google(
        &pool,
        fake_google(profile(&email.to_uppercase(), true)).await,
    );

    let response = sign_in_with_google(&app).await;

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
    let app = app_with_google(&pool, fake_google(owner_profile).await);
    register(&app, &pool, &email, false).await;

    let response = sign_in_with_google(&app).await;

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
