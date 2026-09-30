use std::collections::HashMap;
use std::io::{self, Write};
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::http::StatusCode;
use axum::response::Response;
use clinicore_app::AppState;
use clinicore_core::config::Config;
use clinicore_core::db;
use clinicore_core::mail::Mailer;
use clinicore_core::redis::Redis;
use http_body_util::BodyExt;
use lettre::transport::stub::AsyncStubTransport;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use tracing::subscriber::{DefaultGuard, NoSubscriber};

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
    state(
        config,
        db::connect_lazy(config).expect("a lazy database"),
        AsyncStubTransport::new_ok(),
    )
}

pub fn state(config: &Config, pool: PgPool, mail: AsyncStubTransport) -> AppState {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run");
    state_on_redis(config, pool, mail, &redis_url)
}

pub fn state_on_redis(
    config: &Config,
    pool: PgPool,
    mail: AsyncStubTransport,
    redis_url: &str,
) -> AppState {
    let mut config = config.clone();
    config.redis_url = redis_url.to_string();
    AppState::new(
        &config,
        pool,
        Redis::connect_lazy(&config).expect("a lazy redis"),
        Mailer::stub(&config, mail).expect("a valid sender"),
    )
}

pub const PASSWORD: &str = "Clinica#2026";
pub const MOBILE: (&str, &str) = ("clinicore-client", "mobile");

pub fn fresh_email() -> String {
    static SEQUENCE: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after the epoch")
        .as_nanos();
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("ana.{nanos}.{sequence}@example.com")
}

pub fn app_with(pool: &PgPool) -> Router {
    let config = config_with(&[]);
    clinicore_app::app(
        &config,
        state(&config, pool.clone(), AsyncStubTransport::new_ok()),
    )
}

pub async fn register(app: &Router, pool: &PgPool, email: &str, verified: bool) {
    let body = json!({"name": "Ana Souza", "email": email, "password": PASSWORD});
    let response = post_json(app.clone(), "/users", &body).await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    sqlx::query("UPDATE users SET email_verified = $2 WHERE email = $1")
        .bind(email)
        .bind(verified)
        .execute(pool)
        .await
        .expect("an updated user");
}

pub async fn count(pool: &PgPool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql)
        .fetch_one(pool)
        .await
        .expect("a count")
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

pub async fn post_json(app: Router, path: &str, body: &serde_json::Value) -> Response {
    request(app, "POST", path, &[], Some(body)).await
}

pub async fn request(
    app: Router,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&serde_json::Value>,
) -> Response {
    request_from(fresh_client(), app, method, path, headers, body).await
}

pub async fn request_from(
    client: SocketAddr,
    app: Router,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&serde_json::Value>,
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
    request(app, method, path, &[], None).await
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
    let (pair, _) = set_cookie.split_once(';').unwrap_or((set_cookie, ""));
    let (_, value) = pair.split_once('=').expect("<name>=<value>");
    value
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
    static SHARED_DISPATCHER: Once = Once::new();
    SHARED_DISPATCHER.call_once(|| {
        tracing::subscriber::set_global_default(NoSubscriber::default())
            .expect("no other global subscriber in the test binary");
    });
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
