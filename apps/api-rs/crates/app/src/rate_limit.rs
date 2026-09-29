use std::time::Duration;

use api_infra::redis::Redis;

use crate::error::{AppError, ErrorCode};

#[derive(Clone)]
pub struct RateLimit {
    redis: Redis,
}

impl RateLimit {
    pub fn new(redis: Redis) -> Self {
        Self { redis }
    }

    pub async fn hit(&self, key: &str, limit: u64, window: Duration) -> Result<(), AppError> {
        let hits = self.redis.hit(key, window).await.map_err(|error| {
            tracing::error!(cause = %error, "the rate limit store is unreachable");
            AppError::Business(ErrorCode::ServiceUnavailable)
        })?;
        if hits > limit {
            return Err(AppError::Business(ErrorCode::RateLimited));
        }
        Ok(())
    }
}
