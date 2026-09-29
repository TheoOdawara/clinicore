use crate::support::{CapturedLog, config_with, send};

#[tokio::test]
async fn health_is_not_logged_and_any_other_request_is() {
    let config = config_with(&[("LOG_LEVEL", Some("info"))]);
    let log = CapturedLog::default();
    let writer = log.clone();
    let _subscriber = tracing::subscriber::set_default(clinicore_app::http::telemetry::subscriber(
        &config,
        move || writer.clone(),
    ));

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
    let log = CapturedLog::default();
    let writer = log.clone();
    let _subscriber = tracing::subscriber::set_default(clinicore_app::http::telemetry::subscriber(
        &config,
        move || writer.clone(),
    ));

    tracing::debug!(target: "clinicore_app", "from the api");
    tracing::debug!(target: "hyper", "from a dependency");

    let lines = log.lines();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["message"], "from the api");
}
