use std::net::IpAddr;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use clinicore_core::redis::{Redis, RedisError};
use ipnet::Ipv6Net;

use super::client::ClientAddress;
use super::error::AppError;
use crate::AppState;

#[derive(Clone, Copy)]
pub struct Quota {
    name: &'static str,
    count: u64,
    window: Duration,
}

impl Quota {
    pub const fn new(name: &'static str, count: u64, seconds: u64) -> Self {
        Self {
            name,
            count,
            window: Duration::from_secs(seconds),
        }
    }
}

#[derive(Clone)]
pub struct Limit {
    redis: Redis,
    quota: Quota,
}

impl Limit {
    pub fn new(state: &AppState, quota: Quota) -> Self {
        Self {
            redis: state.redis.clone(),
            quota,
        }
    }
}

pub async fn guard(
    State(limit): State<Limit>,
    client: ClientAddress,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    enforce(&limit.redis, limit.quota, &client_key(client)).await?;
    Ok(next.run(request).await)
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("rate limited")]
    Exceeded,
    #[error(transparent)]
    Redis(#[from] RedisError),
}

impl From<RateLimitError> for AppError {
    fn from(error: RateLimitError) -> Self {
        match error {
            RateLimitError::Exceeded => Self::RateLimited,
            RateLimitError::Redis(error) => Self::Unavailable(error),
        }
    }
}

pub async fn enforce(redis: &Redis, quota: Quota, identity: &str) -> Result<(), RateLimitError> {
    let hits = redis.hit(&key(quota, identity), quota.window).await?;
    if hits > quota.count {
        return Err(RateLimitError::Exceeded);
    }
    Ok(())
}

pub async fn refund(redis: &Redis, quota: Quota, identity: &str) -> Result<(), RedisError> {
    redis.undo_hit(&key(quota, identity)).await
}

fn key(quota: Quota, identity: &str) -> String {
    format!("rate:{}:{identity}", quota.name)
}

fn client_key(ClientAddress(address): ClientAddress) -> String {
    match address {
        None => "unknown".to_string(),
        Some(IpAddr::V6(address)) => Ipv6Net::new_assert(address, 64).trunc().to_string(),
        Some(address) => address.to_string(),
    }
}
