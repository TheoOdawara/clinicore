mod emails;
mod error;
pub(crate) mod handlers;
mod password;
mod queries;
mod requests;
mod service;
mod token;

use axum::Router;
use axum::routing::post;

use crate::AppState;
use crate::http::rate_limit::{self, Limit};

pub fn routes(state: &AppState) -> Router {
    let per_minute = |name, count| {
        axum::middleware::from_fn_with_state(
            Limit::per_minute(state, name, count),
            rate_limit::guard,
        )
    };
    Router::new()
        .route(
            "/users",
            post(handlers::sign_up).route_layer(per_minute("sign-up", 3)),
        )
        .route(
            "/email-verifications",
            post(handlers::request_email_verification)
                .route_layer(per_minute("email-verification", 3)),
        )
        .with_state(state.clone())
}
