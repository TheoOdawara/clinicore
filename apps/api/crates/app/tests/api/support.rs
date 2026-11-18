use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use clinicore_app::{AppState, router};
use clinicore_core::config::{ApiConfig, AppEnv};
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use tracing_subscriber::fmt::MakeWriter;

pub const WEB_ORIGIN: &str = "http://localhost:3000";

pub fn state_with(app_env: AppEnv, overrides: &[(&str, &str)]) -> AppState {
    let app_env = match app_env {
        AppEnv::Development => "development",
        AppEnv::Production => "production",
        AppEnv::Test => "test",
    };
    let mut source = HashMap::from([
        ("APP_ENV", app_env),
        ("LOG_LEVEL", "warn"),
        ("PORT", "3333"),
        (
            "DATABASE_URL",
            "postgresql://clinicore@127.0.0.1:1/clinicore",
        ),
        ("REDIS_URL", "redis://127.0.0.1:1"),
        (
            "ALLOWED_ORIGINS",
            "http://localhost:3000,https://app.clinicore.com.br",
        ),
        ("APP_ORIGIN", WEB_ORIGIN),
        ("CLIENT_IP_SOURCE", "peer"),
        ("JWT_SECRET", "a-test-secret-with-at-least-32-characters"),
    ]);
    source.extend(overrides.iter().copied());
    let config = ApiConfig::from_source(|name| source.get(name).map(|value| (*value).to_owned()))
        .expect("the test environment is valid");
    let db = PgPoolOptions::new()
        .connect_lazy(&config.database_url)
        .expect("the url parses");
    let redis = clinicore_core::redis::connect(&config.redis_url).expect("the url parses");
    AppState {
        db,
        redis,
        config: Arc::new(config),
    }
}

pub fn state() -> AppState {
    state_with(AppEnv::Test, &[])
}

pub fn app(app_env: AppEnv) -> Router {
    router(state_with(app_env, &[]))
}

pub struct Answer {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Answer {
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).expect("the body is JSON")
    }
}

pub async fn send(app: Router, method: Method, path: &str) -> Answer {
    send_with(app, method, path, &[]).await
}

pub async fn send_with(
    app: Router,
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
) -> Answer {
    let mut request = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let request = request
        .body(Body::empty())
        .expect("the request is well formed");
    dispatch(app, request).await
}

pub async fn dispatch(app: Router, request: Request<Body>) -> Answer {
    let response = app
        .oneshot(request)
        .await
        .expect("the router is infallible");
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("the body is readable")
        .to_bytes()
        .to_vec();
    Answer {
        status,
        headers,
        body,
    }
}

#[derive(Clone, Default)]
pub struct Captured(Arc<Mutex<Vec<u8>>>);

impl Captured {
    pub fn text(&self) -> String {
        String::from_utf8(self.0.lock().expect("the buffer is not poisoned").clone())
            .expect("the log is UTF-8")
    }
}

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("the buffer is not poisoned")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Captured {
    type Writer = Captured;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}
