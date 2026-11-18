use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, header};
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;
use crate::http::client::Client;
use crate::http::error::AppError;

pub async fn guard(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    Client::declared(request.headers())?;
    if changes_state(request.method())
        && !is_allowed(request.headers(), &state.config.allowed_origins)
    {
        return Err(AppError::InvalidOrigin);
    }
    Ok(next.run(request).await)
}

fn changes_state(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn is_allowed(headers: &HeaderMap, allowed_origins: &[String]) -> bool {
    match headers.get(header::ORIGIN) {
        Some(origin) => allowed_origins
            .iter()
            .any(|allowed| origin == allowed.as_str()),
        None => !headers.contains_key(header::COOKIE),
    }
}
