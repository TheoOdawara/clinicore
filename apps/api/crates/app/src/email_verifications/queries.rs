use clinicore_core::mail::MailMessage;
use sqlx::PgPool;

use super::error::EmailVerificationError;
use crate::email_dispatches::{EmailDispatchKind, claim_dispatch};
use crate::email_outbox::{VerificationPurpose, enqueue_with_token};

pub async fn request_verification(
    pool: &PgPool,
    email: &str,
    token_hash: &str,
    render: impl FnOnce(&str) -> Result<MailMessage, askama::Error>,
) -> Result<(), EmailVerificationError> {
    let mut transaction = pool.begin().await?;
    if !claim_dispatch(
        &mut transaction,
        email,
        EmailDispatchKind::EmailVerification,
    )
    .await?
    {
        transaction.commit().await?;
        return Ok(());
    }

    let name = sqlx::query_scalar!(
        "SELECT name FROM users WHERE email = $1 AND NOT email_verified",
        email
    )
    .fetch_optional(&mut *transaction)
    .await?;
    if let Some(name) = name {
        enqueue_with_token(
            &mut transaction,
            email,
            VerificationPurpose::EmailVerification,
            token_hash,
            &render(&name)?,
        )
        .await?;
    }
    transaction.commit().await?;
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
