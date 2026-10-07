mod google_sign_in;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use clinicore_app::GoogleEndpoints;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde_json::json;
use tokio::net::TcpListener;

use crate::support::{cookie_value, request, send, set_cookie};

#[derive(Clone, Copy, Debug)]
pub enum IdToken {
    Genuine,
    ForAnotherClient,
    WithAnotherNonce,
    Forged,
}

pub const OAUTH_COOKIES: [&str; 3] = [
    "clinicore_oauth_state",
    "clinicore_oauth_verifier",
    "clinicore_oauth_nonce",
];

pub struct FakeGoogle {
    pub endpoints: GoogleEndpoints,
    nonce: Arc<Mutex<String>>,
}

pub async fn fake_google(id_token: IdToken, profile: serde_json::Value) -> FakeGoogle {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a free local port");
    let address = listener.local_addr().expect("a bound address");
    let endpoints = GoogleEndpoints {
        token: format!("http://{address}/token"),
        keys: format!("http://{address}/keys"),
        ..GoogleEndpoints::production()
    };
    let nonce = Arc::new(Mutex::new(String::new()));

    let issuer = endpoints.issuer.clone();
    let expected_nonce = nonce.clone();
    let token = move |Form(form): Form<HashMap<String, String>>| async move {
        if !form.contains_key("code_verifier") {
            return StatusCode::BAD_REQUEST.into_response();
        }
        let now = jsonwebtoken::get_current_timestamp();
        let mut claims = profile;
        claims["iss"] = json!(issuer);
        claims["aud"] = match id_token {
            IdToken::ForAnotherClient => json!("another-client-id.apps.googleusercontent.com"),
            _ => json!("test-client-id.apps.googleusercontent.com"),
        };
        claims["iat"] = json!(now);
        claims["exp"] = json!(now + 300);
        claims["nonce"] = match id_token {
            IdToken::WithAnotherNonce => json!("the-nonce-of-another-sign-in"),
            _ => json!(*expected_nonce.lock().expect("an unpoisoned nonce")),
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("test-key".to_string());
        let key = EncodingKey::from_rsa_der(include_bytes!("google_test_key.der"));
        let mut signed = jsonwebtoken::encode(&header, &claims, &key).expect("a signed ID token");
        if matches!(id_token, IdToken::Forged) {
            let signature_start = signed.rfind('.').expect("a signature") + 1;
            signed.replace_range(signature_start..signature_start + 8, "AAAAAAAA");
        }
        Json(json!({
            "access_token": "google-access-token",
            "token_type": "Bearer",
            "id_token": signed,
        }))
        .into_response()
    };
    let keys = || async {
        (
            [("content-type", "application/json")],
            include_str!("google_test_keys.json"),
        )
    };
    let router = Router::new()
        .route("/token", post(token))
        .route("/keys", get(keys));
    tokio::spawn(async move { axum::serve(listener, router).await });
    FakeGoogle { endpoints, nonce }
}

pub struct StartedSignIn {
    pub state: String,
    pub cookies: String,
}

pub async fn start(app: &Router, google: &FakeGoogle) -> StartedSignIn {
    let response = send(app.clone(), "GET", "/oauth/google").await;
    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response.headers()["location"]
        .to_str()
        .expect("a text location");
    let parameter = |name: &str| {
        location
            .split(['?', '&'])
            .find_map(|pair| pair.strip_prefix(name))
            .unwrap_or_else(|| panic!("a {name} in the authorization URL"))
            .to_string()
    };
    *google.nonce.lock().expect("an unpoisoned nonce") = parameter("nonce=");
    let cookies = OAUTH_COOKIES
        .map(|name| {
            let cookie = set_cookie(&response, name).unwrap_or_else(|| panic!("a {name} cookie"));
            format!("{name}={}", cookie_value(&cookie))
        })
        .join("; ");
    StartedSignIn {
        state: parameter("state="),
        cookies,
    }
}

pub async fn callback(app: &Router, state: &str, cookies: Option<&str>) -> Response {
    let path = format!("/oauth/google/callback?code=any-code&state={state}");
    let headers: Vec<(&str, &str)> = cookies
        .map(|cookies| ("cookie", cookies))
        .into_iter()
        .collect();
    request(app.clone(), "GET", &path, &headers, None).await
}

pub async fn sign_in_with_google(app: &Router, google: &FakeGoogle) -> Response {
    let started = start(app, google).await;
    callback(app, &started.state, Some(&started.cookies)).await
}
