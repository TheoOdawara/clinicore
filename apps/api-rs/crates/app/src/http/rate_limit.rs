use std::net::IpAddr;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_client_ip::{ClientIp, Rejection};
use clinicore_core::redis::{Redis, RedisError};
use ipnet::Ipv6Net;

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
    client: Result<ClientIp, Rejection>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !admit(&limit.redis, limit.quota, &client_key(client)).await? {
        return Err(AppError::RateLimited);
    }
    Ok(next.run(request).await)
}

pub async fn admit(redis: &Redis, quota: Quota, identity: &str) -> Result<bool, RedisError> {
    let hits = redis.hit(&key(quota, identity), quota.window).await?;
    Ok(hits <= quota.count)
}

pub async fn refund(redis: &Redis, quota: Quota, identity: &str) -> Result<(), RedisError> {
    redis.undo_hit(&key(quota, identity)).await
}

fn key(quota: Quota, identity: &str) -> String {
    format!("rate:{}:{identity}", quota.name)
}

fn client_key(client: Result<ClientIp, Rejection>) -> String {
    let Ok(ClientIp(address)) = client else {
        return "unknown".to_string();
    };
    match address.to_canonical() {
        IpAddr::V6(address) => Ipv6Net::new_assert(address, 64).trunc().to_string(),
        address => address.to_string(),
    }
}
