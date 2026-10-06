mod cookies;
mod error;
pub(crate) mod google;
pub(crate) mod handlers;
mod queries;
mod requests;
mod service;

use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouterExt};
use utoipa_axum::routes;

use crate::AppState;
use crate::http::rate_limit::{self, Limit, Quota};

const NETWORK_REQUESTS: Quota = Quota::new("oauth-google-network", 300, 60);

pub fn routes(state: &AppState) -> OpenApiRouter {
    let limit = |name| {
        axum::middleware::from_fn_with_state(
            Limit::new(state, Quota::new(name, 10, 60)),
            rate_limit::guard,
        )
    };
    OpenApiRouter::new()
        .routes(
            routes!(handlers::start_google_sign_in)
                .map(|router| router.route_layer(limit("oauth-google"))),
        )
        .routes(
            routes!(handlers::complete_google_sign_in)
                .map(|router| router.route_layer(limit("oauth-google-callback"))),
        )
        .with_state(state.clone())
}
