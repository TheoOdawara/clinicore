use lettre::transport::stub::AsyncStubTransport;
use sqlx::PgPool;

use crate::sessions::Transport;
use crate::support::{PASSWORD, capture_log, config_with, register, send, state};

#[tokio::test]
async fn health_is_not_logged_and_any_other_request_is() {
    let config = config_with(&[("LOG_LEVEL", Some("info"))]);
    let (log, _subscriber) = capture_log(&config);

    send(crate::support::app(&config), "GET", "/health").await;
    send(crate::support::app(&config), "HEAD", "/health").await;
    send(
        crate::support::app(&config),
        "GET",
        "/rota-inexistente?secret=1",
    )
    .await;

    let lines = log.lines();
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line = &lines[0];
    assert_eq!(line["span"]["method"], "GET");
    assert_eq!(line["span"]["path"], "/rota-inexistente");
    assert_eq!(line["status"], 404);
    assert!(line["duration_ms"].as_f64().is_some(), "{line}");
}

#[test]
fn log_level_debug_reaches_the_api_crates_and_not_the_dependencies() {
    let config = config_with(&[("LOG_LEVEL", Some("debug"))]);
    let (log, _subscriber) = capture_log(&config);

    tracing::debug!(target: "clinicore_app", "from the api");
    tracing::debug!(target: "hyper", "from a dependency");

    let lines = log.lines();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["message"], "from the api");
}

#[sqlx::test(migrations = "../../migrations")]
async fn trace_logs_the_session_requests_without_any_secret(pool: PgPool) {
    let config = config_with(&[("LOG_LEVEL", Some("trace"))]);
    let (log, _subscriber) = capture_log(&config);
    let app = clinicore_app::app(
        &config,
        state(&config, pool.clone(), AsyncStubTransport::new_ok()),
    );
    register(&app, &pool, "ana@example.com", true).await;

    let web = Transport::Web.sign_in(&app, "ana@example.com").await;
    let mobile = Transport::Mobile.sign_in(&app, "ana@example.com").await;
    Transport::Web
        .call(&app, "GET", "/sessions/current", &web.access)
        .await;
    let rotated = Transport::Mobile
        .session_from(Transport::Mobile.refresh(&app, &mobile.refresh).await)
        .await;

    let lines = log.lines();
    let sign_in = lines
        .iter()
        .find(|line| line["span"]["path"] == "/sessions" && line["status"] == 201)
        .unwrap_or_else(|| panic!("a sign-in line in {lines:?}"));
    assert_eq!(sign_in["span"]["method"], "POST");
    assert!(sign_in["duration_ms"].as_f64().is_some(), "{sign_in}");

    let written = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL, injected by infisical run");
    for secret in [
        PASSWORD,
        &web.access,
        &web.refresh,
        &mobile.access,
        &mobile.refresh,
        &rotated.access,
        &rotated.refresh,
        &database_url,
        config.database_url.as_str(),
    ] {
        assert!(!written.contains(secret), "a secret reached the log");
    }
}
