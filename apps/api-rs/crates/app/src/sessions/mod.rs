pub(crate) mod error;
mod extractors;
pub(crate) mod handlers;
mod queries;
mod requests;
mod responses;
mod service;
mod tokens;

use axum::Router;
use axum::routing::{get, post};

pub(crate) use tokens::access::AccessKeys;

use crate::AppState;
use crate::http::rate_limit::{self, Limit, Quota};

const SIGN_IN_FAILURES: Quota = Quota::new("sign-in-failures", 10, 15 * 60);
const SESSION_REQUESTS: Quota = Quota::new("session", 100, 10);
const SESSION_REFRESHES: Quota = Quota::new("session-refresh", 30, 60);

pub fn routes(state: &AppState) -> Router {
    let limit = |name, count, seconds| {
        axum::middleware::from_fn_with_state(
            Limit::new(state, Quota::new(name, count, seconds)),
            rate_limit::guard,
        )
    };
    Router::new()
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
