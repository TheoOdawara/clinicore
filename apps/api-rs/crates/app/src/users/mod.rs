mod error;
pub(crate) mod handlers;
mod queries;
mod requests;
mod service;

use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouterExt};
use utoipa_axum::routes;

use crate::AppState;
use crate::http::rate_limit::{self, Limit, Quota};

pub fn routes(state: &AppState) -> OpenApiRouter {
    let limit = axum::middleware::from_fn_with_state(
        Limit::new(state, Quota::new("sign-up", 3, 60)),
        rate_limit::guard,
    );
    OpenApiRouter::new()
        .routes(routes!(handlers::sign_up).map(|router| router.route_layer(limit)))
        .with_state(state.clone())
}
