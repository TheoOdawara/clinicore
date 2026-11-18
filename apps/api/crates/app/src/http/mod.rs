pub mod client;
pub mod client_ip;
pub mod cors;
pub mod error;
pub mod log;
pub mod openapi;
pub mod origin;

use axum::Router;
use axum::http::{HeaderValue, header};
use axum::middleware;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::AppState;

pub fn layers(router: Router<AppState>, state: AppState) -> Router {
    let cors = cors::layer(&state.config.allowed_origins);
    router
        .route_layer(middleware::from_fn_with_state(state.clone(), origin::guard))
        .fallback(error::not_found)
        .method_not_allowed_fallback(error::method_not_allowed)
        .layer(cors)
        .layer(CatchPanicLayer::custom(error::panic_response))
        .layer(log::layer())
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .with_state(state)
}
