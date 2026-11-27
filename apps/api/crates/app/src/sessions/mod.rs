pub mod cookies;
pub mod extractors;
pub mod handlers;
pub mod requests;
pub mod responses;
pub mod service;
pub mod tokens;

mod queries;

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionId(pub Uuid);

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::create))
        .routes(routes!(handlers::show, handlers::delete))
        .routes(routes!(handlers::refresh))
}
