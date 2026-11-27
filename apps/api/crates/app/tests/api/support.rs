use std::collections::HashMap;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use clinicore_app::{AppState, router};
use clinicore_core::config::{ApiConfig, AppEnv};
use http_body_util::BodyExt;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use tracing_subscriber::fmt::MakeWriter;

pub const WEB_ORIGIN: &str = "http://localhost:3000";

fn config(app_env: AppEnv, overrides: &[(&str, &str)]) -> ApiConfig {
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
    ApiConfig::from_source(|name| source.get(name).map(|value| (*value).to_owned()))
        .expect("the test environment is valid")
}

pub fn state_with(app_env: AppEnv, overrides: &[(&str, &str)]) -> AppState {
    let config = config(app_env, overrides);
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

pub fn state_on(db: PgPool, overrides: &[(&str, &str)]) -> AppState {
    let redis_url = std::env::var("REDIS_URL").expect("the tests run with REDIS_URL");
    let mut overrides = overrides.to_vec();
    if !overrides.iter().any(|(name, _)| *name == "REDIS_URL") {
        overrides.push(("REDIS_URL", redis_url.as_str()));
    }
    let config = config(AppEnv::Test, &overrides);
    let redis = clinicore_core::redis::connect(&config.redis_url).expect("the url parses");
    AppState {
        db,
        redis,
        config: Arc::new(config),
    }
}

pub fn app_on(db: PgPool) -> Router {
    router(state_on(db, &[]))
}

pub fn random_ip() -> IpAddr {
    IpAddr::V4(Ipv4Addr::from(rand::random::<[u8; 4]>()))
}

pub async fn post_json(
    app: Router,
    path: &str,
    body: &serde_json::Value,
    headers: &[(&str, &str)],
) -> Answer {
    dispatch(
        app,
        from_peer(json_request(path, body, headers), random_ip()),
    )
    .await
}

pub fn json_request(
    path: &str,
    body: &serde_json::Value,
    headers: &[(&str, &str)],
) -> Request<Body> {
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    request
        .body(Body::from(body.to_string()))
        .expect("the request is well formed")
}

pub fn from_peer(mut request: Request<Body>, peer: IpAddr) -> Request<Body> {
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(peer, 50_000)));
    request
}

pub fn state() -> AppState {
    state_with(AppEnv::Test, &[])
}

pub fn app(app_env: AppEnv) -> Router {
    router(state_with(app_env, &[]))
}

pub async fn count(pool: &PgPool, table: &str) -> i64 {
    let sql = match table {
        "users" => "SELECT count(*) FROM users",
        "accounts" => "SELECT count(*) FROM accounts",
        "verifications" => "SELECT count(*) FROM verifications",
        "email_dispatches" => "SELECT count(*) FROM email_dispatches",
        "email_outbox" => "SELECT count(*) FROM email_outbox",
        "sessions" => "SELECT count(*) FROM sessions",
        _ => panic!("{table} is not an authentication table"),
    };
    sqlx::query_scalar(sql)
        .fetch_one(pool)
        .await
        .expect("the table is readable")
}

pub async fn counts(pool: &PgPool) -> [i64; 5] {
    [
        count(pool, "users").await,
        count(pool, "accounts").await,
        count(pool, "verifications").await,
        count(pool, "email_dispatches").await,
        count(pool, "email_outbox").await,
    ]
}

pub fn token_in(text: &str) -> String {
    let prefix = format!("{WEB_ORIGIN}/verify-email?token=");
    text.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .expect("the text carries the link on its own line")
        .to_owned()
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
    dispatch(app, empty_request(method, path, headers)).await
}

pub async fn send_from_anywhere(
    app: Router,
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
) -> Answer {
    dispatch(
        app,
        from_peer(empty_request(method, path, headers), random_ip()),
    )
    .await
}

fn empty_request(method: Method, path: &str, headers: &[(&str, &str)]) -> Request<Body> {
    let mut request = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    request
        .body(Body::empty())
        .expect("the request is well formed")
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

pub const PASSWORD: &str = "Clinica#2026";

pub fn fresh_email(name: &str) -> String {
    format!("{name}-{:016x}@exemplo.com", rand::random::<u64>())
}

pub async fn account(pool: &PgPool, email: &str, verified: bool) -> uuid::Uuid {
    let password_hash = clinicore_app::credentials::password::hash(PASSWORD.to_owned()).await;
    let id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO users (name, email, email_verified) VALUES ('Ana Souza', $1, $2) RETURNING id",
    )
    .bind(email)
    .bind(verified)
    .fetch_one(pool)
    .await
    .expect("the user is inserted");
    sqlx::query(
        "INSERT INTO accounts (user_id, provider, password_hash) VALUES ($1, 'credential', $2)",
    )
    .bind(id)
    .bind(password_hash)
    .execute(pool)
    .await
    .expect("the account is inserted");
    id
}

pub async fn sign_in(app: Router, email: &str, password: &str, headers: &[(&str, &str)]) -> Answer {
    post_json(
        app,
        "/sessions",
        &serde_json::json!({ "email": email, "password": password }),
        headers,
    )
    .await
}

pub fn cookie(answer: &Answer, name: &str) -> String {
    answer
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().expect("a cookie is ascii"))
        .find(|value| value.starts_with(&format!("{name}=")))
        .unwrap_or_else(|| panic!("the answer sets {name}"))
        .to_owned()
}

pub fn cookie_value(answer: &Answer, name: &str) -> String {
    let cookie = cookie(answer, name);
    let pair = cookie.split(';').next().expect("the cookie has a pair");
    pair[name.len() + 1..].to_owned()
}

pub struct WebSession {
    pub access: String,
    pub refresh: String,
}

pub async fn web_session(app: Router, email: &str) -> WebSession {
    let answer = sign_in(app, email, PASSWORD, &[("origin", WEB_ORIGIN)]).await;
    assert_eq!(answer.status, StatusCode::CREATED);
    WebSession {
        access: cookie_value(&answer, "clinicore_access"),
        refresh: cookie_value(&answer, "clinicore_refresh"),
    }
}

pub struct MobileSession {
    pub access: String,
    pub refresh: String,
}

pub async fn mobile_session(app: Router, email: &str) -> MobileSession {
    let answer = sign_in(app, email, PASSWORD, &[("clinicore-client", "mobile")]).await;
    assert_eq!(answer.status, StatusCode::CREATED);
    let tokens = &answer.json()["tokens"];
    MobileSession {
        access: tokens["accessToken"].as_str().expect("a string").to_owned(),
        refresh: tokens["refreshToken"]
            .as_str()
            .expect("a string")
            .to_owned(),
    }
}

pub fn session_id_of(refresh_token: &str) -> uuid::Uuid {
    refresh_token
        .split_once('.')
        .and_then(|(id, _)| id.parse().ok())
        .expect("the refresh token starts with the session id")
}

pub async fn current_as_web(app: Router, method: Method, access: &str) -> Answer {
    let cookie = format!("clinicore_access={access}");
    send_from_anywhere(
        app,
        method,
        "/sessions/current",
        &[("cookie", cookie.as_str()), ("origin", WEB_ORIGIN)],
    )
    .await
}

pub async fn current_as_mobile(app: Router, method: Method, access: &str) -> Answer {
    let bearer = format!("Bearer {access}");
    send_from_anywhere(
        app,
        method,
        "/sessions/current",
        &[
            ("authorization", bearer.as_str()),
            ("clinicore-client", "mobile"),
        ],
    )
    .await
}

pub async fn refresh_as_web(app: Router, refresh: &str) -> Answer {
    let cookie = format!("clinicore_refresh={refresh}");
    send_from_anywhere(
        app,
        Method::POST,
        "/sessions/current/tokens",
        &[("cookie", cookie.as_str()), ("origin", WEB_ORIGIN)],
    )
    .await
}

pub async fn refresh_as_mobile(app: Router, refresh: &str) -> Answer {
    post_json(
        app,
        "/sessions/current/tokens",
        &serde_json::json!({ "refreshToken": refresh }),
        &[("clinicore-client", "mobile")],
    )
    .await
}

pub fn assert_problem(answer: &Answer, status: StatusCode, slug: &str) {
    assert_eq!(
        answer.status,
        status,
        "{:?}",
        String::from_utf8_lossy(&answer.body)
    );
    assert_eq!(
        answer.json()["type"],
        format!("tag:clinicore.com.br,2026:{slug}")
    );
}
