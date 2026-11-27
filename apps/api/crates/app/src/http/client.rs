use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};

use crate::http::error::AppError;

pub const CLIENT_HEADER: &str = "clinicore-client";
const MOBILE: &str = "mobile";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "session_client", rename_all = "snake_case")]
pub enum Client {
    Web,
    Mobile,
}

impl Client {
    pub fn declared(headers: &HeaderMap) -> Result<Self, AppError> {
        match headers.get(CLIENT_HEADER) {
            None => Ok(Self::Web),
            Some(value) if value == MOBILE => Ok(Self::Mobile),
            Some(_) => Err(AppError::InvalidClient),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Client {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Self::declared(&parts.headers)
    }
}
