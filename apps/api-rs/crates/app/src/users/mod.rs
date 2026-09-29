pub(crate) mod handlers;
mod queries;
mod requests;
mod service;

use axum::Router;
use axum::routing::post;

use crate::AppState;
use crate::http::rate_limit::{self, Limit};

pub fn routes(state: &AppState) -> Router {
    let limit = axum::middleware::from_fn_with_state(
        Limit::per_minute(state, "sign-up", 3),
        rate_limit::guard,
    );
    Router::new()
        .route("/users", post(handlers::sign_up).route_layer(limit))
        .with_state(state.clone())
}
