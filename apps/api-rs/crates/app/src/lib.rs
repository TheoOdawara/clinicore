pub use api_infra::config;
pub use api_infra::db::Database;
pub use api_infra::mail::Mailer;
pub use api_infra::redis::Redis;

pub mod auth;
pub mod error;
pub mod rate_limit;

use config::Config;

#[derive(Clone)]
pub struct Services {
    pub auth: auth::Auth,
    pub rate_limit: rate_limit::RateLimit,
}

impl Services {
    pub fn new(config: &Config, database: Database, redis: Redis, mailer: Mailer) -> Self {
        Self {
            auth: auth::Auth::new(config, database, mailer),
            rate_limit: rate_limit::RateLimit::new(redis),
        }
    }
}
