use std::time::Duration;

use api_app::Services;
use api_app::rate_limit::RateLimit;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_client_ip::{ClientIp, Rejection};

use crate::client_ip;
use crate::error::ApiError;

#[derive(Clone)]
pub struct Limit {
    rate_limit: RateLimit,
    route: &'static str,
    count: u64,
    window: Duration,
}

impl Limit {
    pub fn per_minute(services: &Services, route: &'static str, count: u64) -> Self {
        Self {
            rate_limit: services.rate_limit.clone(),
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
) -> Result<Response, ApiError> {
    let key = format!("rate:{}:{}", limit.route, client_ip::tracker(client));
    limit
        .rate_limit
        .hit(&key, limit.count, limit.window)
        .await
        .map_err(ApiError)?;
    Ok(next.run(request).await)
}
