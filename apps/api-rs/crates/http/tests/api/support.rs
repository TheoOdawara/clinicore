use std::collections::HashMap;
use std::io::{self, Write};
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use api_app::config::Config;
use api_app::{Database, Mailer, Redis, Services};
use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::response::Response;
use http_body_util::BodyExt;
use lettre::transport::stub::AsyncStubTransport;
use sqlx::PgPool;
use tower::ServiceExt;

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

pub fn lazy_services(config: &Config) -> Services {
    Services::new(
        config,
        Database::connect_lazy(config).expect("a lazy database"),
        Redis::connect_lazy(config).expect("a lazy redis"),
        Mailer::stub(config, AsyncStubTransport::new_ok()).expect("a valid sender"),
    )
}

pub fn services(config: &Config, pool: PgPool, mail: AsyncStubTransport) -> Services {
    let mut config = config.clone();
    config.redis_url = std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run");
    Services::new(
        &config,
        Database::from_pool(pool),
        Redis::connect_lazy(&config).expect("a lazy redis"),
        Mailer::stub(&config, mail).expect("a valid sender"),
    )
}

pub fn app(config: &Config) -> Router {
    api_http::app(config, lazy_services(config))
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
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .extension(ConnectInfo(client))
        .body(Body::from(body.to_string()))
        .expect("a valid request");
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
