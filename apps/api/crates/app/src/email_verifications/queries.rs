use sqlx::{PgExecutor, PgPool};

use super::error::EmailVerificationError;

pub struct UserSummary {
    pub name: String,
    pub email_verified: bool,
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserSummary>, sqlx::Error> {
    sqlx::query_as!(
        UserSummary,
        "SELECT name, email_verified FROM users WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_verification(
    executor: impl PgExecutor<'_>,
    email: &str,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO verifications (email, purpose, token_hash, expires_at)
        VALUES ($1, 'email_verification', $2, now() + interval '1 hour')",
        email,
        token_hash
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn consume_email_verification(
    pool: &PgPool,
    token_hash: &str,
) -> Result<(), EmailVerificationError> {
    let mut transaction = pool.begin().await?;
    let email = sqlx::query_scalar!(
        "SELECT email FROM verifications WHERE token_hash = $1 AND purpose = 'email_verification'",
        token_hash
    )
    .fetch_optional(&mut *transaction)
    .await?;
    let Some(email) = email else {
        return Err(EmailVerificationError::InvalidToken);
    };

    sqlx::query!(
        r#"SELECT true AS "locked!" FROM pg_advisory_xact_lock(hashtext('email_verifications'), hashtext($1))"#,
        email
    )
    .fetch_one(&mut *transaction)
    .await?;

    let live = sqlx::query_scalar!(
        r#"SELECT expires_at > now() AS "live!"
        FROM verifications
        WHERE token_hash = $1 AND purpose = 'email_verification' AND consumed_at IS NULL"#,
        token_hash
    )
    .fetch_optional(&mut *transaction)
    .await?;
    match live {
        None => return Err(EmailVerificationError::InvalidToken),
        Some(false) => return Err(EmailVerificationError::TokenExpired),
        Some(true) => {}
    }

    let verified = sqlx::query!(
        "UPDATE users SET email_verified = true, updated_at = now() WHERE email = $1",
        email
    )
    .execute(&mut *transaction)
    .await?;
    if verified.rows_affected() == 0 {
        return Err(EmailVerificationError::InvalidToken);
    }

    sqlx::query!(
        "UPDATE verifications SET consumed_at = now()
        WHERE email = $1 AND purpose = 'email_verification' AND consumed_at IS NULL",
        email
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(())
}
