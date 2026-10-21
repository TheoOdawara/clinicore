mod error;
pub(crate) mod handlers;
mod queries;
mod requests;
mod service;

use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouterExt};
use utoipa_axum::routes;

use crate::AppState;
use crate::http::rate_limit::{self, Limit, Quota};

const PASSWORD_FAILURES: Quota = Quota::new("password-change-failures", 10, 15 * 60);

pub fn routes(state: &AppState) -> OpenApiRouter {
    let limit = |name| {
        axum::middleware::from_fn_with_state(
            Limit::new(state, Quota::new(name, 3, 60)),
            rate_limit::guard,
        )
    };
    OpenApiRouter::new()
        .routes(routes!(handlers::sign_up).map(|router| router.route_layer(limit("sign-up"))))
        .routes(
            routes!(handlers::change_password)
                .map(|router| router.route_layer(limit("password-change"))),
        )
        .with_state(state.clone())
}
