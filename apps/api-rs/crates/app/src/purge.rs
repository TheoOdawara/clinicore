use std::time::Duration;

use sqlx::PgPool;

use crate::AppState;
use crate::email_verifications::queries::DISPATCH_WINDOW;
use crate::sessions::tokens::refresh;

const EVERY: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, PartialEq, Eq)]
pub struct Purged {
    pub sessions: u64,
    pub verifications: u64,
    pub email_dispatches: u64,
}

// ponytail: every API replica runs its own hourly purge, harmless because the deletes are idempotent; moves to crates/worker when #71 creates it
pub async fn run(state: AppState) {
    let mut interval = tokio::time::interval(EVERY);
    loop {
        interval.tick().await;
        match purge_expired(&state.pool).await {
            Ok(purged) => tracing::info!(
                sessions = purged.sessions,
                verifications = purged.verifications,
                email_dispatches = purged.email_dispatches,
                "purged the expired authentication records"
            ),
            Err(error) => {
                tracing::error!(error = %error, "could not purge the expired authentication records")
            }
        }
    }
}

pub async fn purge_expired(pool: &PgPool) -> Result<Purged, sqlx::Error> {
    let sessions = sqlx::query!(
        "DELETE FROM sessions
        WHERE expires_at <= now() OR created_at <= now() - make_interval(secs => $1)",
        refresh::ABSOLUTE_LIFETIME.as_secs_f64()
    )
    .execute(pool)
    .await?;
    let verifications = sqlx::query!("DELETE FROM verifications WHERE expires_at <= now()")
        .execute(pool)
        .await?;
    let email_dispatches = sqlx::query!(
        "DELETE FROM email_dispatches WHERE created_at <= now() - make_interval(secs => $1)",
        DISPATCH_WINDOW.as_secs_f64()
    )
    .execute(pool)
    .await?;
    Ok(Purged {
        sessions: sessions.rows_affected(),
        verifications: verifications.rows_affected(),
        email_dispatches: email_dispatches.rows_affected(),
    })
}
