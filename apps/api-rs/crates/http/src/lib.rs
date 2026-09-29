pub mod error;
pub mod telemetry;

mod auth;
mod client_ip;
mod health;
mod middleware;
mod openapi;
mod validation;

use api_app::Services;
use api_app::config::{AppEnv, Config};
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::Response;
use axum::routing::get;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn app(config: &Config, services: Services) -> Router {
    serve_layers(routes(config, services), config)
}

pub fn routes(config: &Config, services: Services) -> Router {
    let router = Router::new()
        .route("/health", get(health::check))
        .merge(auth::routes(&services));
    if config.app_env == AppEnv::Production {
        return router;
    }
    router.merge(openapi::docs())
}

pub fn serve_layers(router: Router, config: &Config) -> Router {
    let allowed_origins = config.allowed_origins.iter().map(|origin| {
        HeaderValue::from_str(origin).expect("a validated origin is a valid header value")
    });
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE]);
    let origin_guard = axum::middleware::from_fn_with_state(
        middleware::origin::AllowedOrigins::from(config),
        middleware::origin::guard,
    );

    router
        .route_layer(origin_guard)
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .layer(
            ServiceBuilder::new()
                .layer(telemetry::request_log())
                .layer(cors)
                .layer(CatchPanicLayer::custom(error::panic_response))
                .layer(config.client_ip_source.clone().into_extension())
                .layer(DefaultBodyLimit::max(100 * 1024)),
        )
}

async fn not_found() -> Response {
    error::blank_problem(StatusCode::NOT_FOUND)
}
