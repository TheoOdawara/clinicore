use redis::aio::ConnectionManager;

use crate::http::error::AppError;
use crate::sessions::SessionId;
use crate::sessions::tokens::refresh::LONGEST_LIFETIME;

pub async fn revoke(redis: &ConnectionManager, session_id: SessionId) -> Result<(), AppError> {
    let mut connection = redis.clone();
    redis::cmd("SET")
        .arg(key(session_id))
        .arg(1)
        .arg("EX")
        .arg(LONGEST_LIFETIME.as_secs())
        .query_async::<()>(&mut connection)
        .await
        .map_err(|error| unavailable(&error))
}

pub async fn is_revoked(
    redis: &ConnectionManager,
    session_id: SessionId,
) -> Result<bool, AppError> {
    let mut connection = redis.clone();
    redis::cmd("EXISTS")
        .arg(key(session_id))
        .query_async::<bool>(&mut connection)
        .await
        .map_err(|error| unavailable(&error))
}

pub fn key(session_id: SessionId) -> String {
    format!("auth:revoked:{}", session_id.0)
}

fn unavailable(error: &redis::RedisError) -> AppError {
    tracing::error!(%error, "could not reach the session denylist");
    AppError::ServiceUnavailable
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn an_unreachable_redis_is_a_service_unavailable() {
        let redis = clinicore_core::redis::connect("redis://127.0.0.1:1").expect("the url parses");
        let session_id = SessionId(Uuid::from_u128(1));

        assert!(matches!(
            revoke(&redis, session_id).await,
            Err(AppError::ServiceUnavailable)
        ));
        assert!(matches!(
            is_revoked(&redis, session_id).await,
            Err(AppError::ServiceUnavailable)
        ));
    }
}
