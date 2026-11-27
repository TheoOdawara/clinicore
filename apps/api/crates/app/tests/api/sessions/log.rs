use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;

use crate::support::{
    Captured, PASSWORD, WEB_ORIGIN, app_on, cookie_value, current_as_mobile, current_as_web,
    fresh_email, post_json, refresh_as_mobile, refresh_as_web, sign_in, token_in,
};

#[sqlx::test(migrations = "../../migrations")]
async fn no_secret_reaches_the_log(pool: PgPool) {
    let captured = Captured::default();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::TRACE)
        .with_current_span(true)
        .with_writer(captured.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    let email = fresh_email("ana");
    let mut secrets = vec![PASSWORD.to_owned()];

    let signed_up = post_json(
        app_on(pool.clone()),
        "/users",
        &json!({ "name": "Ana Souza", "email": email, "password": PASSWORD }),
        &[],
    )
    .await;
    assert_eq!(signed_up.status, StatusCode::ACCEPTED);
    let text: String = sqlx::query_scalar("SELECT text_body FROM email_outbox")
        .fetch_one(&pool)
        .await
        .expect("the link is queued");
    let link_token = token_in(&text);
    let confirmed = post_json(
        app_on(pool.clone()),
        "/email-verifications/confirmation",
        &json!({ "token": link_token }),
        &[],
    )
    .await;
    assert_eq!(confirmed.status, StatusCode::NO_CONTENT);
    secrets.push(link_token);

    let web = sign_in(
        app_on(pool.clone()),
        &email,
        PASSWORD,
        &[("origin", WEB_ORIGIN)],
    )
    .await;
    let web_access = cookie_value(&web, "clinicore_access");
    let web_refresh = cookie_value(&web, "clinicore_refresh");
    let rotated = refresh_as_web(app_on(pool.clone()), &web_refresh).await;
    let rotated_access = cookie_value(&rotated, "clinicore_access");
    let signed_out = current_as_web(app_on(pool.clone()), Method::DELETE, &rotated_access).await;
    assert_eq!(signed_out.status, StatusCode::NO_CONTENT);

    let mobile = sign_in(
        app_on(pool.clone()),
        &email,
        PASSWORD,
        &[("clinicore-client", "mobile")],
    )
    .await;
    let tokens = mobile.json()["tokens"].clone();
    let mobile_access = tokens["accessToken"].as_str().expect("a string").to_owned();
    let mobile_refresh = tokens["refreshToken"]
        .as_str()
        .expect("a string")
        .to_owned();
    let read = current_as_mobile(app_on(pool.clone()), Method::GET, &mobile_access).await;
    assert_eq!(read.status, StatusCode::OK);
    let renewed = refresh_as_mobile(app_on(pool.clone()), &mobile_refresh).await;
    assert_eq!(renewed.status, StatusCode::OK);
    let wrong = sign_in(app_on(pool), &email, "Errada#2026", &[]).await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);

    let log = captured.text();
    assert!(
        log.contains(r#""path":"/sessions""#),
        "the requests are logged"
    );
    secrets.extend([
        web_access,
        web_refresh,
        rotated_access,
        cookie_value(&rotated, "clinicore_refresh"),
        mobile_access,
        mobile_refresh,
        "Errada#2026".to_owned(),
        "Bearer".to_owned(),
        "clinicore_access".to_owned(),
        "clinicore_refresh".to_owned(),
    ]);
    for secret in secrets {
        assert!(!log.contains(&secret), "the log carries {secret}");
    }
}
