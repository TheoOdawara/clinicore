pub mod mail;
pub mod outbox;
pub mod purge;
pub mod schedule;

use sqlx::PgPool;
use tokio::sync::watch;

use crate::mail::Mailer;

pub async fn run(pool: PgPool, mailer: Mailer, stop: watch::Receiver<bool>) {
    tokio::join!(
        outbox::run(pool.clone(), mailer, stop.clone()),
        schedule::run(pool, stop),
    );
}
