mod access_token;
mod cookies;
pub(crate) mod emails;
pub(crate) mod error;
pub(crate) mod handlers;
pub(crate) mod password;
pub(crate) mod queries;
mod requests;
mod responses;
pub(crate) mod service;
mod session;
pub(crate) mod token;

use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};

pub(crate) use access_token::AccessKeys;

use crate::AppState;
use crate::http::rate_limit::{self, Limit};

pub fn routes(state: &AppState) -> Router {
    let limit = |name, count, seconds| {
        axum::middleware::from_fn_with_state(
            Limit::new(state, name, count, Duration::from_secs(seconds)),
            rate_limit::guard,
        )
    };
    Router::new()
        .route(
            "/email-verifications",
            post(handlers::request_email_verification).route_layer(limit(
                "email-verification",
                3,
                60,
            )),
        )
        .route(
            "/email-verifications/confirmation",
            post(handlers::confirm_email).route_layer(limit("email-confirmation", 100, 10)),
        )
        .route(
            "/sessions",
            post(handlers::sign_in).route_layer(limit("sign-in", 5, 60)),
        )
        .route(
            "/sessions/current",
            get(handlers::current)
                .delete(handlers::sign_out)
                .route_layer(limit("current-session", 100, 10)),
        )
        .route(
            "/sessions/current/tokens",
            post(handlers::refresh).route_layer(limit("token-refresh", 30, 60)),
        )
        .with_state(state.clone())
}
