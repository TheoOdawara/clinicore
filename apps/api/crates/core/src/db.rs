use std::time::Duration;

use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPoolOptions;
use sqlx::postgres::types::PgInterval;

const FIRST_RETRY: Duration = Duration::from_secs(1);
const LONGEST_RETRY: Duration = Duration::from_secs(30);
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum BootError {
    #[error("the database refused the connection: {0}")]
    Database(#[source] sqlx::Error),
    #[error("a migration failed: {0}")]
    Migration(#[from] MigrateError),
}

pub async fn connect_and_migrate(url: &str, migrator: &Migrator) -> Result<PgPool, BootError> {
    let mut retry_in = FIRST_RETRY;
    let pool = loop {
        match PgPoolOptions::new()
            .acquire_timeout(ACQUIRE_TIMEOUT)
            .connect(url)
            .await
        {
            Ok(pool) => break pool,
            Err(error @ (sqlx::Error::Io(_) | sqlx::Error::PoolTimedOut)) => {
                tracing::warn!(%error, retry_in_secs = retry_in.as_secs(), "database unavailable");
                tokio::time::sleep(retry_in).await;
                retry_in = (retry_in * 2).min(LONGEST_RETRY);
            }
            Err(error) => return Err(BootError::Database(error)),
        }
    };
    migrator.run(&pool).await?;
    Ok(pool)
}

pub fn interval(duration: Duration) -> PgInterval {
    PgInterval::try_from(duration).expect("every lifetime the services use fits an interval")
}
