use std::convert::Infallible;
use std::net::IpAddr;

use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use axum_client_ip::ClientIp;
use serde::{Deserialize, Serialize};

use super::error::AppError;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "session_client", rename_all = "lowercase")]
pub enum SessionClient {
    Web,
    Mobile,
}

impl<State: Send + Sync> FromRequestParts<State> for SessionClient {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &State) -> Result<Self, AppError> {
        match parts.headers.get("clinicore-client") {
            None => Ok(Self::Web),
            Some(value) if value == "mobile" => Ok(Self::Mobile),
            Some(_) => Err(AppError::InvalidClient),
        }
    }
}

pub async fn guard(_: SessionClient, request: Request, next: Next) -> Response {
    next.run(request).await
}

pub struct ClientAddress(pub Option<IpAddr>);

impl<State: Send + Sync> FromRequestParts<State> for ClientAddress {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Infallible> {
        let address = ClientIp::from_request_parts(parts, state)
            .await
            .ok()
            .map(|ClientIp(address)| address.to_canonical());
        Ok(Self(address))
    }
}
