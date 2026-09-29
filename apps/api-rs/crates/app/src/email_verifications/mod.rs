pub(crate) mod emails;
pub(crate) mod error;
pub(crate) mod handlers;
pub(crate) mod queries;
mod requests;
pub(crate) mod service;

use axum::Router;
use axum::routing::post;

use crate::AppState;
use crate::http::rate_limit::{self, Limit, Quota};

const EMAIL_CONFIRMATIONS: Quota = Quota::new("email-confirmation-total", 300, 60);

pub fn routes(state: &AppState) -> Router {
    let limit = |name, count, seconds| {
        axum::middleware::from_fn_with_state(
            Limit::new(state, Quota::new(name, count, seconds)),
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
        .with_state(state.clone())
}
