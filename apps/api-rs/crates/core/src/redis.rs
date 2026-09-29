use std::time::Duration;

pub use redis::RedisError;
use redis::RedisResult;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};

use crate::config::Config;

#[derive(Clone)]
pub struct Redis {
    connection: ConnectionManager,
}

impl Redis {
    pub fn connect_lazy(config: &Config) -> RedisResult<Self> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let settings = ConnectionManagerConfig::new()
            .set_connection_timeout(Some(Duration::from_secs(1)))
            .set_response_timeout(Some(Duration::from_secs(1)))
            .set_number_of_retries(1);
        let connection = ConnectionManager::new_lazy_with_config(client, settings)?;
        Ok(Self { connection })
    }

    pub async fn hit(&self, key: &str, window: Duration) -> RedisResult<u64> {
        let mut connection = self.connection.clone();
        let (hits,): (u64,) = redis::pipe()
            .atomic()
            .incr(key, 1)
            .cmd("EXPIRE")
            .arg(key)
            .arg(window.as_secs())
            .arg("NX")
            .ignore()
            .query_async(&mut connection)
            .await?;
        Ok(hits)
    }
}
