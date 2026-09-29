pub(crate) mod error;
mod extractors;
pub(crate) mod handlers;
mod queries;
mod requests;
mod responses;
mod service;
pub(crate) mod tokens;

use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouterExt};
use utoipa_axum::routes;

pub(crate) use tokens::access::AccessKeys;

use crate::AppState;
use crate::http::rate_limit::{self, Limit, Quota};

const SIGN_IN_FAILURES: Quota = Quota::new("sign-in-failures", 10, 15 * 60);
const SESSION_REQUESTS: Quota = Quota::new("session", 100, 10);
const SESSION_REFRESHES: Quota = Quota::new("session-refresh", 30, 60);

pub fn routes(state: &AppState) -> OpenApiRouter {
    let limit = |name, count, seconds| {
        axum::middleware::from_fn_with_state(
            Limit::new(state, Quota::new(name, count, seconds)),
            rate_limit::guard,
        )
    };
    OpenApiRouter::new()
        .routes(
            routes!(handlers::sign_in).map(|router| router.route_layer(limit("sign-in", 5, 60))),
        )
        .routes(
            routes!(handlers::read_current_session)
                .map(|router| router.route_layer(limit("current-session", 100, 10))),
        )
        .routes(
            routes!(handlers::sign_out)
                .map(|router| router.route_layer(limit("sign-out", 100, 10))),
        )
        .routes(
            routes!(handlers::refresh)
                .map(|router| router.route_layer(limit("token-refresh", 30, 60))),
        )
        .with_state(state.clone())
}
