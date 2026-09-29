use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_client_ip::{ClientIp, Rejection};
use clinicore_core::redis::Redis;

use crate::AppState;
use crate::client_ip;
use crate::error::{AppError, ErrorCode};

#[derive(Clone)]
pub struct Limit {
    redis: Redis,
    route: &'static str,
    count: u64,
    window: Duration,
}

impl Limit {
    pub fn per_minute(state: &AppState, route: &'static str, count: u64) -> Self {
        Self {
            redis: state.redis.clone(),
            route,
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
    let key = format!("rate:{}:{}", limit.route, client_ip::tracker(client));
    let hits = limit.redis.hit(&key, limit.window).await.map_err(|error| {
        tracing::error!(cause = %error, "the rate limit store is unreachable");
        AppError::Business(ErrorCode::ServiceUnavailable)
    })?;
    if hits > limit.count {
        return Err(AppError::Business(ErrorCode::RateLimited));
    }
    Ok(next.run(request).await)
}
