use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::Method;
use axum::http::header::{COOKIE, ORIGIN};
use axum::middleware::Next;
use axum::response::Response;
use clinicore_core::config::Config;

use super::error::AppError;

#[derive(Clone)]
pub struct AllowedOrigins(Arc<Vec<String>>);

impl From<&Config> for AllowedOrigins {
    fn from(config: &Config) -> Self {
        Self(Arc::new(config.allowed_origins.clone()))
    }
}

pub async fn guard(
    State(allowed): State<AllowedOrigins>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if [Method::GET, Method::HEAD, Method::OPTIONS].contains(request.method()) {
        return Ok(next.run(request).await);
    }

    let origin = request.headers().get(ORIGIN);
    if origin.is_none() && !request.headers().contains_key(COOKIE) {
        return Ok(next.run(request).await);
    }

    let is_allowed = origin
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| allowed.0.iter().any(|item| item == origin));
    if !is_allowed {
        return Err(AppError::InvalidOrigin);
    }

    Ok(next.run(request).await)
}
