use std::marker::PhantomData;
use std::time::Duration;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use redis::aio::ConnectionManager;

use crate::AppState;
use crate::http::client_ip::ClientIp;
use crate::http::error::AppError;

pub struct Rule {
    pub name: &'static str,
    pub window: Duration,
    pub max: u64,
}

pub trait Policy {
    const RULES: &'static [Rule];
}

pub struct Limited<P>(PhantomData<P>);

impl<P: Policy> FromRequestParts<AppState> for Limited<P> {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Ok(client_ip) = ClientIp::from_request_parts(parts, state).await;
        for rule in P::RULES {
            let key = format!("rate:{}:ip:{}", rule.name, client_ip.key());
            hit(&state.redis, &key, rule.window, rule.max).await?;
        }
        Ok(Self(PhantomData))
    }
}

pub async fn hit(
    redis: &ConnectionManager,
    key: &str,
    window: Duration,
    max: u64,
) -> Result<(), AppError> {
    let mut connection = redis.clone();
    let (count, ttl): (u64, i64) = redis::pipe()
        .atomic()
        .cmd("SET")
        .arg(key)
        .arg(0)
        .arg("EX")
        .arg(window.as_secs())
        .arg("NX")
        .ignore()
        .incr(key, 1)
        .ttl(key)
        .query_async(&mut connection)
        .await
        .map_err(|error| {
            tracing::error!(%error, "could not count the request against its limit");
            AppError::ServiceUnavailable
        })?;
    if count > max {
        return Err(AppError::RateLimited {
            retry_after: u64::try_from(ttl).unwrap_or(0).max(1),
        });
    }
    Ok(())
}
