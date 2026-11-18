pub mod http;

mod health;

use std::sync::Arc;

use axum::Router;
use clinicore_core::config::{ApiConfig, AppEnv};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub config: Arc<ApiConfig>,
}

pub fn router(state: AppState) -> Router {
    let (routes, openapi) = OpenApiRouter::with_openapi(http::openapi::ApiDoc::openapi())
        .merge(health::routes())
        .split_for_parts();
    let routes = match state.config.app_env {
        AppEnv::Production => routes,
        AppEnv::Development | AppEnv::Test => {
            routes.merge(SwaggerUi::new("/api").url("/api-json", openapi))
        }
    };
    http::layers(routes, state)
}
