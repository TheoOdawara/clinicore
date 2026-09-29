#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use api_app::config::Config;
use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use http_body_util::BodyExt;
use tower::ServiceExt;

pub const VALID_ENVIRONMENT: [(&str, &str); 17] = [
    ("ALLOWED_ORIGINS", "http://localhost:3000"),
    ("API_URL", "http://localhost:3333"),
    ("APP_ENV", "test"),
    ("APP_ORIGIN", "http://localhost:3000"),
    ("DATABASE_URL", "postgresql://app:local@localhost:5432/app"),
    (
        "GOOGLE_CLIENT_ID",
        "test-client-id.apps.googleusercontent.com",
    ),
    ("GOOGLE_CLIENT_SECRET", "test-google-client-secret"),
    (
        "JWT_SECRET",
        "a-test-jwt-secret-with-at-least-32-characters",
    ),
    ("LOG_LEVEL", "info"),
    ("MAIL_FROM", "test@example.com"),
    ("PORT", "3333"),
    ("REDIS_URL", "redis://localhost:6379"),
    ("SMTP_HOST", "smtp.gmail.com"),
    ("SMTP_PASSWORD", "test-smtp-password"),
    ("SMTP_PORT", "587"),
    ("SMTP_USER", "test@example.com"),
    ("TRUSTED_PROXIES", "10.0.0.0/8"),
];

pub fn environment_with(overrides: &[(&str, Option<&str>)]) -> HashMap<String, String> {
    let mut environment: HashMap<String, String> = VALID_ENVIRONMENT
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    for (name, value) in overrides {
        match value {
            Some(value) => environment.insert(name.to_string(), value.to_string()),
            None => environment.remove(*name),
        };
    }
    environment
}

pub fn config_with(overrides: &[(&str, Option<&str>)]) -> Config {
    let environment = environment_with(overrides);
    match Config::from_source(|name| environment.get(name).cloned()) {
        Ok(config) => config,
        Err(rejection) => panic!("the test environment is invalid:\n{rejection}"),
    }
}

pub async fn send(app: Router, method: &str, path: &str) -> Response {
    send_with_headers(app, method, path, &[]).await
}

pub async fn send_with_headers(
    app: Router,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
) -> Response {
    let mut request = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let request = request.body(Body::empty()).expect("a valid request");
    app.oneshot(request).await.expect("an infallible router")
}

pub async fn text_body(response: Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("a readable body")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("a UTF-8 body")
}

pub async fn json_body(response: Response) -> serde_json::Value {
    serde_json::from_str(&text_body(response).await).expect("a JSON body")
}

pub fn content_type(response: &Response) -> &str {
    response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

#[derive(Clone, Default)]
pub struct CapturedLog(Arc<Mutex<Vec<u8>>>);

impl CapturedLog {
    pub fn lines(&self) -> Vec<serde_json::Value> {
        let bytes = self.0.lock().expect("an unpoisoned log").clone();
        String::from_utf8(bytes)
            .expect("a UTF-8 log")
            .lines()
            .map(|line| serde_json::from_str(line).expect("a JSON log line"))
            .collect()
    }
}

impl Write for CapturedLog {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("an unpoisoned log")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
