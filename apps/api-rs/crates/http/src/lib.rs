pub mod error;
pub mod telemetry;

mod health;
mod openapi;
mod origin;

use api_app::config::{AppEnv, Config};
use axum::Router;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::middleware;
use axum::response::Response;
use axum::routing::get;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn app(config: &Config) -> Router {
    serve_layers(routes(config), config)
}

pub fn routes(config: &Config) -> Router {
    let router = Router::new().route("/health", get(health::check));
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
    let origin_guard =
        middleware::from_fn_with_state(origin::AllowedOrigins::from(config), origin::guard);

    router
        .route_layer(origin_guard)
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .layer(
            ServiceBuilder::new()
                .layer(telemetry::request_log())
                .layer(cors)
                .layer(CatchPanicLayer::custom(error::panic_response)),
        )
}

async fn not_found() -> Response {
    error::blank_problem(StatusCode::NOT_FOUND)
}
