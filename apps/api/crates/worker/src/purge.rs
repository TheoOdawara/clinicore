use clinicore_core::db::interval;
use clinicore_core::lifetimes::{DISPATCH_WINDOW, SESSION_ABSOLUTE_LIFETIME};
use sqlx::PgPool;

const PURGED: &str = "purged the expired authentication records";

#[derive(Clone, Copy)]
struct Purged {
    sessions: u64,
    verifications: u64,
    email_dispatches: u64,
    email_outbox: u64,
}

pub async fn run(pool: &PgPool) {
    match purge(pool).await {
        Ok(purged) => log(&purged),
        Err(error) => {
            tracing::error!(%error, "could not purge the expired authentication records");
        }
    }
}

#[rustfmt::skip]
fn log(purged: &Purged) {
    let Purged { sessions, verifications, email_dispatches, email_outbox } = *purged;
    tracing::info!(sessions, verifications, email_dispatches, email_outbox, "{PURGED}");
}

async fn purge(pool: &PgPool) -> Result<Purged, sqlx::Error> {
    let sessions = sqlx::query!(
        "DELETE FROM sessions WHERE expires_at <= now() OR created_at <= now() - $1::interval",
        interval(SESSION_ABSOLUTE_LIFETIME),
    )
    .execute(pool)
    .await?;
    let verifications = sqlx::query!("DELETE FROM verifications WHERE expires_at <= now()")
        .execute(pool)
        .await?;
    let email_dispatches = sqlx::query!(
        "DELETE FROM email_dispatches WHERE created_at <= now() - $1::interval",
        interval(DISPATCH_WINDOW),
    )
    .execute(pool)
    .await?;
    let email_outbox = sqlx::query!("DELETE FROM email_outbox WHERE expires_at <= now()")
        .execute(pool)
        .await?;
    Ok(Purged {
        sessions: sessions.rows_affected(),
        verifications: verifications.rows_affected(),
        email_dispatches: email_dispatches.rows_affected(),
        email_outbox: email_outbox.rows_affected(),
    })
}
