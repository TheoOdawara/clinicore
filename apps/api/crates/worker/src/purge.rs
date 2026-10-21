use clinicore_core::lifetimes::{DISPATCH_WINDOW, SESSION_ABSOLUTE_LIFETIME};
use sqlx::PgPool;

#[derive(Debug, PartialEq, Eq)]
pub struct Purged {
    pub sessions: u64,
    pub verifications: u64,
    pub email_dispatches: u64,
}

pub async fn purge_and_log(pool: &PgPool) {
    match purge_expired(pool).await {
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

pub async fn purge_expired(pool: &PgPool) -> Result<Purged, sqlx::Error> {
    let sessions = sqlx::query!(
        "DELETE FROM sessions
        WHERE expires_at <= now() OR created_at <= now() - make_interval(secs => $1)",
        SESSION_ABSOLUTE_LIFETIME.as_secs_f64()
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
