mod emails;
mod error;
pub(crate) mod handlers;
mod queries;
mod requests;
mod service;

use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouterExt};
use utoipa_axum::routes;

use crate::AppState;
use crate::http::rate_limit::{self, Limit, Quota};

const NETWORK_CONFIRMATIONS: Quota = Quota::new("password-reset-confirmation-network", 300, 60);

pub fn routes(state: &AppState) -> OpenApiRouter {
    let limit = |name| {
        axum::middleware::from_fn_with_state(
            Limit::new(state, Quota::new(name, 5, 60)),
            rate_limit::guard,
        )
    };
    OpenApiRouter::new()
        .routes(
            routes!(handlers::request_password_reset)
                .map(|router| router.route_layer(limit("password-reset"))),
        )
        .routes(
            routes!(handlers::confirm_password_reset)
                .map(|router| router.route_layer(limit("password-reset-confirmation"))),
        )
        .with_state(state.clone())
}
