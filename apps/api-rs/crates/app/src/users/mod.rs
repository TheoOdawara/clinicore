pub(crate) mod handlers;
mod queries;
mod requests;
mod service;

use std::time::Duration;

use axum::Router;
use axum::routing::post;

use crate::AppState;
use crate::http::rate_limit::{self, Limit};

pub fn routes(state: &AppState) -> Router {
    let limit = axum::middleware::from_fn_with_state(
        Limit::new(state, "sign-up", 3, Duration::from_secs(60)),
        rate_limit::guard,
    );
    Router::new()
        .route("/users", post(handlers::sign_up).route_layer(limit))
        .with_state(state.clone())
}
