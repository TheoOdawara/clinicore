use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use clinicore_app::{AppState, router};
use clinicore_core::config::AppEnv;
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

pub fn app(app_env: AppEnv) -> Router {
    let unreachable = PgPoolOptions::new()
        .connect_lazy("postgresql://clinicore@127.0.0.1:1/clinicore")
        .expect("the url parses");
    router(AppState { db: unreachable }, app_env)
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
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .expect("the request is well formed");
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
