use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

pub fn connect_lazy(options: &PgConnectOptions) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(options.clone())
}
