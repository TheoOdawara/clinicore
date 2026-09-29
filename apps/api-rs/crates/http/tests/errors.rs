mod common;

use api_app::error::AppError;
use api_http::error::ApiError;
use axum::http::StatusCode;
use axum::routing::get;
use common::{CapturedLog, config_with, content_type, json_body, send_with_headers};
use serde_json::json;

async fn panicking() -> &'static str {
    panic!("the panicking cause")
}

async fn failing() -> Result<&'static str, ApiError> {
    Err(ApiError(AppError::Internal("the internal cause".into())))
}

#[tokio::test]
async fn an_unhandled_error_answers_500_without_detail_through_cors_and_logs_the_cause() {
    let config = config_with(&[("LOG_LEVEL", Some("error"))]);
    let log = CapturedLog::default();
    let writer = log.clone();
    let _subscriber =
        tracing::subscriber::set_default(api_http::telemetry::subscriber(&config, move || {
            writer.clone()
        }));
    let app = api_http::serve_layers(
        api_http::routes(&config)
            .route("/boom", get(panicking))
            .route("/boom-internal", get(failing)),
        &config,
    );

    for (path, cause) in [
        ("/boom", "the panicking cause"),
        ("/boom-internal", "the internal cause"),
    ] {
        let response = send_with_headers(
            app.clone(),
            "GET",
            path,
            &[("origin", "http://localhost:3000")],
        )
        .await;

        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{path}"
        );
        assert!(content_type(&response).starts_with("application/problem+json"));
        assert_eq!(
            response.headers()["access-control-allow-origin"],
            "http://localhost:3000",
            "{path}"
        );
        assert_eq!(
            json_body(response).await,
            json!({"type": "about:blank", "title": "Internal Server Error", "status": 500})
        );
        assert!(
            log.lines()
                .iter()
                .any(|line| line["level"] == "ERROR" && line.to_string().contains(cause)),
            "{path}: {:?}",
            log.lines()
        );
    }
}
