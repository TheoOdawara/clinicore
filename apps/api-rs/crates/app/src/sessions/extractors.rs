use std::net::IpAddr;

use axum::extract::FromRequestParts;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum::http::request::Parts;
use axum_client_ip::ClientIp;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

use super::SESSION_REQUESTS;
use super::error::SessionError;
use super::tokens::{access, cookies};
use crate::AppState;
use crate::http::client::SessionClient;
use crate::http::error::AppError;
use crate::http::rate_limit;

pub struct CurrentSession {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

impl FromRequestParts<AppState> for CurrentSession {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let client = SessionClient::from_request_parts(parts, state).await?;
        let token = match client {
            SessionClient::Web => CookieJar::from_headers(&parts.headers)
                .get(cookies::ACCESS)
                .map(|cookie| cookie.value().to_string()),
            SessionClient::Mobile => parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .map(str::to_string),
        };
        let Some(token) = token else {
            return Err(AppError::InvalidSession);
        };
        let claims = access::verify(&state.access_keys, &token)
            .map_err(SessionError::from)?
            .filter(|claims| claims.cli == client)
            .ok_or(AppError::InvalidSession)?;
        if access::is_revoked(&state.redis, claims.sid).await? {
            return Err(AppError::InvalidSession);
        }
        rate_limit::enforce(&state.redis, SESSION_REQUESTS, &claims.sid.to_string()).await?;
        Ok(Self {
            user_id: claims.sub,
            session_id: claims.sid,
        })
    }
}

pub struct Device {
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
}

impl<State: Send + Sync> FromRequestParts<State> for Device {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, AppError> {
        let ip_address = ClientIp::from_request_parts(parts, state)
            .await
            .ok()
            .map(|ClientIp(address)| address.to_canonical());
        let user_agent = parts
            .headers
            .get(USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.chars().take(512).collect());
        Ok(Self {
            ip_address,
            user_agent,
        })
    }
}
