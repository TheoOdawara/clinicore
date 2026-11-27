pub mod handlers;
pub mod requests;
pub mod service;

mod queries;

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(pub Uuid);

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(handlers::create))
}
