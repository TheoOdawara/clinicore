use axum::http::HeaderMap;

use crate::http::error::AppError;

pub const CLIENT_HEADER: &str = "clinicore-client";
const MOBILE: &str = "mobile";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
