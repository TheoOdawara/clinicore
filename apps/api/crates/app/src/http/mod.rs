pub mod error;
pub mod log;
pub mod openapi;

use axum::Router;
use tower_http::catch_panic::CatchPanicLayer;

pub fn layers<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .fallback(error::not_found)
        .method_not_allowed_fallback(error::method_not_allowed)
        .layer(CatchPanicLayer::custom(error::panic_response))
        .layer(log::layer())
}
