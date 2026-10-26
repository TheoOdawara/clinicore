use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

const STATEMENT_TIMEOUT: &str = "10s";
const LOCK_TIMEOUT: &str = "3s";

pub fn connect_lazy(options: &PgConnectOptions) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(options.clone().options([
            ("statement_timeout", STATEMENT_TIMEOUT),
            ("lock_timeout", LOCK_TIMEOUT),
        ]))
}
