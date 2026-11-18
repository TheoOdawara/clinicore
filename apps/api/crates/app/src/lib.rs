pub mod http;

mod health;

use axum::Router;
use clinicore_core::config::AppEnv;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn router(state: AppState, app_env: AppEnv) -> Router {
    let (routes, openapi) = OpenApiRouter::with_openapi(http::openapi::ApiDoc::openapi())
        .merge(health::routes())
        .split_for_parts();
    let routes = match app_env {
        AppEnv::Production => routes,
        AppEnv::Development | AppEnv::Test => {
            routes.merge(SwaggerUi::new("/api").url("/api-json", openapi))
        }
    };
    http::layers(routes).with_state(state)
}
