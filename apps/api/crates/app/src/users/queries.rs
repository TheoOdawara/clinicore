use clinicore_core::redis::Redis;
use sqlx::PgPool;
use uuid::Uuid;

use super::error::UserError;
use crate::email_dispatches::{EmailDispatchKind, claim_dispatch};
use crate::email_verifications::queries::create_verification;
use crate::sessions::extractors::CurrentSession;
use crate::sessions::tokens::access;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignUpOutcome {
    CreatedWithToken,
    Created,
    Duplicate,
}

pub async fn create_user(
    pool: &PgPool,
    name: &str,
    email: &str,
    password_hash: &str,
    token_hash: &str,
) -> Result<SignUpOutcome, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let created = sqlx::query!(
        "WITH created AS (
            INSERT INTO users (name, email) VALUES ($1, $2)
            ON CONFLICT (email) DO NOTHING
            RETURNING id
        )
        INSERT INTO accounts (user_id, provider, password_hash)
        SELECT id, 'credential', $3 FROM created",
        name,
        email,
        password_hash
    )
    .execute(&mut *transaction)
    .await?;

    if created.rows_affected() == 0 {
        return Ok(SignUpOutcome::Duplicate);
    }

    if !claim_dispatch(
        &mut transaction,
        email,
        EmailDispatchKind::EmailVerification,
    )
    .await?
    {
        transaction.commit().await?;
        return Ok(SignUpOutcome::Created);
    }

    create_verification(&mut *transaction, email, token_hash).await?;
    transaction.commit().await?;
    Ok(SignUpOutcome::CreatedWithToken)
}

pub async fn find_password_hash(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT password_hash AS "password_hash!" FROM accounts
        WHERE user_id = $1 AND provider = 'credential'"#,
        user_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn change_password(
    pool: &PgPool,
    redis: &Redis,
    session: &CurrentSession,
    current_hash: &str,
    new_hash: &str,
) -> Result<(), UserError> {
    let mut transaction = pool.begin().await?;
    sqlx::query!(
        "SELECT id FROM users WHERE id = $1 FOR UPDATE",
        session.user_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    let changed = sqlx::query!(
        "UPDATE accounts SET password_hash = $3, updated_at = now()
        WHERE user_id = $1 AND provider = 'credential' AND password_hash = $2",
        session.user_id,
        current_hash,
        new_hash
    )
    .execute(&mut *transaction)
    .await?;
    if changed.rows_affected() == 0 {
        return Err(UserError::InvalidPassword);
    }

    let revoked = sqlx::query_scalar!(
        "DELETE FROM sessions WHERE user_id = $1 AND id <> $2 RETURNING id",
        session.user_id,
        session.session_id
    )
    .fetch_all(&mut *transaction)
    .await?;

    access::revoke(redis, &revoked).await?;
    transaction.commit().await?;
    Ok(())
}
