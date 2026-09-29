use std::collections::HashMap;
use std::io::{self, Write};
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::response::Response;
use clinicore_app::AppState;
use clinicore_core::config::Config;
use clinicore_core::db;
use clinicore_core::mail::Mailer;
use clinicore_core::redis::Redis;
use http_body_util::BodyExt;
use lettre::transport::stub::AsyncStubTransport;
use sqlx::PgPool;
use tower::ServiceExt;
use tracing::subscriber::DefaultGuard;

pub const VALID_ENVIRONMENT: [(&str, &str); 17] = [
    ("ALLOWED_ORIGINS", "http://localhost:3000"),
    ("API_URL", "http://localhost:3333"),
    ("APP_ENV", "test"),
    ("APP_ORIGIN", "http://localhost:3000"),
    ("CLIENT_IP_SOURCE", "ConnectInfo"),
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

pub fn lazy_state(config: &Config) -> AppState {
    AppState::new(
        config,
        db::connect_lazy(config).expect("a lazy database"),
        Redis::connect_lazy(config).expect("a lazy redis"),
        Mailer::stub(config, AsyncStubTransport::new_ok()).expect("a valid sender"),
    )
    .expect("an unmatchable hash")
}

pub fn state(config: &Config, pool: PgPool, mail: AsyncStubTransport) -> AppState {
    let mut config = config.clone();
    config.redis_url = std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run");
    AppState::new(
        &config,
        pool,
        Redis::connect_lazy(&config).expect("a lazy redis"),
        Mailer::stub(&config, mail).expect("a valid sender"),
    )
    .expect("an unmatchable hash")
}

pub fn app(config: &Config) -> Router {
    clinicore_app::app(config, lazy_state(config))
}

pub fn fresh_client() -> SocketAddr {
    static SEQUENCE: AtomicU16 = AtomicU16::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after the epoch")
        .subsec_nanos();
    let address = Ipv6Addr::new(
        0x2001,
        0xdb8,
        (nanos >> 16) as u16,
        (nanos as u16) ^ SEQUENCE.fetch_add(1, Ordering::Relaxed),
        0,
        0,
        0,
        1,
    );
    SocketAddr::from((address, 40000))
}

pub async fn post_json(
    app: Router,
    path: &str,
    body: &serde_json::Value,
    client: SocketAddr,
) -> Response {
    request(app, "POST", path, &[], Some(body), client).await
}

pub async fn request(
    app: Router,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&serde_json::Value>,
    client: SocketAddr,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .extension(ConnectInfo(client));
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let body = match body {
        Some(body) => {
            request = request.header("content-type", "application/json");
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };
    let request = request.body(body).expect("a valid request");
    app.oneshot(request).await.expect("an infallible router")
}

pub async fn eventually<Check, Ready>(mut check: Check) -> bool
where
    Check: FnMut() -> Ready,
    Ready: Future<Output = bool>,
{
    for _ in 0..100 {
        if check().await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    false
}

pub async fn send(app: Router, method: &str, path: &str) -> Response {
    request(app, method, path, &[], None, fresh_client()).await
}

pub fn set_cookie(response: &Response, name: &str) -> Option<String> {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with(&format!("{name}=")))
        .map(str::to_string)
}

pub fn cookie_value(set_cookie: &str) -> &str {
    let pair = set_cookie.split(';').next().unwrap_or_default();
    pair.split_once('=').map_or("", |(_, value)| value)
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

pub fn capture_log(config: &Config) -> (CapturedLog, DefaultGuard) {
    let log = CapturedLog::default();
    let writer = log.clone();
    let guard =
        tracing::subscriber::set_default(clinicore_app::telemetry::subscriber(config, move || {
            writer.clone()
        }));
    (log, guard)
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
