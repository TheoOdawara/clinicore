pub(crate) mod emails;
pub(crate) mod error;
pub(crate) mod handlers;
pub(crate) mod queries;
mod requests;
pub(crate) mod service;

use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouterExt};
use utoipa_axum::routes;

use crate::AppState;
use crate::http::rate_limit::{self, Limit, Quota};

const EMAIL_CONFIRMATIONS: Quota = Quota::new("email-confirmation-total", 300, 60);

pub fn routes(state: &AppState) -> OpenApiRouter {
    let limit = |name, count, seconds| {
        axum::middleware::from_fn_with_state(
            Limit::new(state, Quota::new(name, count, seconds)),
            rate_limit::guard,
        )
    };
    OpenApiRouter::new()
        .routes(
            routes!(handlers::request_email_verification)
                .map(|router| router.route_layer(limit("email-verification", 3, 60))),
        )
        .routes(
            routes!(handlers::confirm_email)
                .map(|router| router.route_layer(limit("email-confirmation", 100, 10))),
        )
        .with_state(state.clone())
}
