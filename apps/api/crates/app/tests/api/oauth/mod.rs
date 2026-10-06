mod google_sign_in;

use std::collections::HashMap;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use clinicore_app::GoogleEndpoints;
use serde_json::json;
use tokio::net::TcpListener;

use crate::support::{cookie_value, request, send, set_cookie};

pub async fn fake_google(profile: serde_json::Value) -> GoogleEndpoints {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a free local port");
    let address = listener.local_addr().expect("a bound address");
    let router = Router::new()
        .route(
            "/token",
            post(|Form(form): Form<HashMap<String, String>>| async move {
                if !form.contains_key("code_verifier") {
                    return StatusCode::BAD_REQUEST.into_response();
                }
                Json(json!({"access_token": "google-access-token", "token_type": "Bearer"}))
                    .into_response()
            }),
        )
        .route("/userinfo", get(move || async move { Json(profile) }));
    tokio::spawn(async move { axum::serve(listener, router).await });
    GoogleEndpoints {
        token: format!("http://{address}/token"),
        userinfo: format!("http://{address}/userinfo"),
        ..GoogleEndpoints::production()
    }
}

pub struct StartedSignIn {
    pub state: String,
    pub cookies: String,
}

pub async fn start(app: &Router) -> StartedSignIn {
    let response = send(app.clone(), "GET", "/oauth/google").await;
    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response.headers()["location"]
        .to_str()
        .expect("a text location");
    let state = location
        .split(['?', '&'])
        .find_map(|pair| pair.strip_prefix("state="))
        .expect("a state in the authorization URL")
        .to_string();
    let cookies = ["clinicore_oauth_state", "clinicore_oauth_verifier"]
        .map(|name| {
            let cookie = set_cookie(&response, name).unwrap_or_else(|| panic!("a {name} cookie"));
            format!("{name}={}", cookie_value(&cookie))
        })
        .join("; ");
    StartedSignIn { state, cookies }
}

pub async fn callback(app: &Router, state: &str, cookies: Option<&str>) -> Response {
    let path = format!("/oauth/google/callback?code=any-code&state={state}");
    let headers: Vec<(&str, &str)> = cookies
        .map(|cookies| ("cookie", cookies))
        .into_iter()
        .collect();
    request(app.clone(), "GET", &path, &headers, None).await
}

pub async fn sign_in_with_google(app: &Router) -> Response {
    let started = start(app).await;
    callback(app, &started.state, Some(&started.cookies)).await
}
