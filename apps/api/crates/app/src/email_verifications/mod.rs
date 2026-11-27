pub mod handlers;
pub mod requests;
pub mod service;

mod queries;

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::create))
        .routes(routes!(handlers::confirm))
}
