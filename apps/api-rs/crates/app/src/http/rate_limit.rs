use std::net::IpAddr;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_client_ip::{ClientIp, Rejection};
use clinicore_core::redis::Redis;
use ipnet::Ipv6Net;

use super::error::AppError;
use crate::AppState;

#[derive(Clone)]
pub struct Limit {
    redis: Redis,
    name: &'static str,
    count: u64,
    window: Duration,
}

impl Limit {
    pub fn per_minute(state: &AppState, name: &'static str, count: u64) -> Self {
        Self {
            redis: state.redis.clone(),
            name,
            count,
            window: Duration::from_secs(60),
        }
    }
}

pub async fn guard(
    State(limit): State<Limit>,
    client: Result<ClientIp, Rejection>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = format!("rate:{}:{}", limit.name, client_key(client));
    let hits = limit.redis.hit(&key, limit.window).await?;
    if hits > limit.count {
        return Err(AppError::RateLimited);
    }
    Ok(next.run(request).await)
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
