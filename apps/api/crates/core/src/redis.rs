use std::time::Duration;

use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use redis::{Client, RedisResult};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(2);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(1);
const RECONNECTION_RETRIES: usize = 1;

pub fn connect(url: &str) -> RedisResult<ConnectionManager> {
    let config = ConnectionManagerConfig::new()
        .set_connection_timeout(Some(CONNECTION_TIMEOUT))
        .set_response_timeout(Some(RESPONSE_TIMEOUT))
        .set_number_of_retries(RECONNECTION_RETRIES);
    ConnectionManager::new_lazy_with_config(Client::open(url)?, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_redis_url_connects_lazily_and_a_malformed_one_is_refused() {
        assert!(connect("redis://127.0.0.1:1").is_ok());
        assert!(connect("not a url").is_err());
    }
}
