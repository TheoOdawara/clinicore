use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use serde::{Deserialize, Serialize};

use super::error::AppError;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "session_client", rename_all = "lowercase")]
pub enum SessionClient {
    Web,
    Mobile,
}

pub async fn guard(mut request: Request, next: Next) -> Result<Response, AppError> {
    let client = match request.headers().get("clinicore-client") {
        None => SessionClient::Web,
        Some(value) if value == "mobile" => SessionClient::Mobile,
        Some(_) => return Err(AppError::InvalidClient),
    };
    request.extensions_mut().insert(client);
    Ok(next.run(request).await)
}

impl<State: Send + Sync> FromRequestParts<State> for SessionClient {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &State) -> Result<Self, AppError> {
        parts
            .extensions
            .get::<SessionClient>()
            .copied()
            .ok_or_else(|| AppError::Internal("the client guard did not run".into()))
    }
}
