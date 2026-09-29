pub mod http;
pub mod telemetry;

mod auth;
mod health;
mod users;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::routing::get;
use clinicore_core::config::{AppEnv, Config};
use clinicore_core::mail::Mailer;
use clinicore_core::redis::Redis;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    redis: Redis,
    mailer: Mailer,
    app_origin: String,
}

impl AppState {
    pub fn new(config: &Config, pool: PgPool, redis: Redis, mailer: Mailer) -> Self {
        Self {
            pool,
            redis,
            mailer,
            app_origin: config.app_origin.clone(),
        }
    }
}

pub fn app(config: &Config, state: AppState) -> Router {
    with_layers(routes(config, state), config)
}

pub fn routes(config: &Config, state: AppState) -> Router {
    let router = Router::new()
        .route("/health", get(health::check))
        .merge(users::routes(&state))
        .merge(auth::routes(&state));
    if config.app_env == AppEnv::Production {
        return router;
    }
    router.merge(http::openapi::routes())
}

pub fn with_layers(router: Router, config: &Config) -> Router {
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
        http::origin::AllowedOrigins::from(config),
        http::origin::guard,
    );

    router
        .route_layer(origin_guard)
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .layer(
            ServiceBuilder::new()
                .layer(http::request_log::layer())
                .layer(cors)
                .layer(CatchPanicLayer::custom(http::error::panic_response))
                .layer(config.client_ip_source.clone().into_extension())
                .layer(DefaultBodyLimit::max(100 * 1024)),
        )
}

async fn not_found() -> http::error::AppError {
    http::error::AppError::Rejected(StatusCode::NOT_FOUND)
}
