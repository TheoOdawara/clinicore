use std::time::Duration;

use axum::http::{HeaderName, HeaderValue, Method, header};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::http::client::CLIENT_HEADER;

const PREFLIGHT_MAX_AGE: Duration = Duration::from_secs(600);

pub fn layer(allowed_origins: &[String]) -> CorsLayer {
    let origins = allowed_origins.iter().map(|origin| {
        HeaderValue::from_str(origin)
            .expect("the config accepts only origins that are header values")
    });
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static(CLIENT_HEADER),
        ])
        .max_age(PREFLIGHT_MAX_AGE)
}
