use std::net::IpAddr;
use std::time::Duration;

use axum::extract::FromRequestParts;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum::http::request::Parts;
use axum_client_ip::ClientIp;
use axum_extra::extract::cookie::CookieJar;
use clinicore_core::redis::{Redis, RedisError};
use uuid::Uuid;

use super::{access_token, cookies};
use crate::AppState;
use crate::http::client::SessionClient;
use crate::http::error::AppError;

pub struct CurrentSession {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub client: SessionClient,
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
        let claims = token
            .and_then(|token| access_token::verify(&state.access_keys, &token))
            .filter(|claims| claims.cli == client)
            .ok_or(AppError::InvalidSession)?;
        if state.redis.exists(&revoked_key(claims.sid)).await? {
            return Err(AppError::InvalidSession);
        }
        Ok(Self {
            user_id: claims.sub,
            session_id: claims.sid,
            client,
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

pub fn lifetime(client: SessionClient) -> Duration {
    match client {
        SessionClient::Web => Duration::from_secs(24 * 60 * 60),
        SessionClient::Mobile => Duration::from_secs(7 * 24 * 60 * 60),
    }
}

pub async fn revoke(redis: &Redis, session_ids: &[Uuid]) -> Result<(), RedisError> {
    let keys: Vec<String> = session_ids.iter().copied().map(revoked_key).collect();
    redis.set_expiring(&keys, access_token::LIFETIME).await
}

fn revoked_key(session_id: Uuid) -> String {
    format!("auth:revoked:{session_id}")
}
