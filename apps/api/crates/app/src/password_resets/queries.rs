use clinicore_core::redis::Redis;
use sqlx::PgPool;

use super::error::PasswordResetError;
use crate::sessions::tokens::access;

pub async fn find_name(pool: &PgPool, email: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!("SELECT name FROM users WHERE email = $1", email)
        .fetch_optional(pool)
        .await
}

pub async fn create_password_reset(
    pool: &PgPool,
    email: &str,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO verifications (email, purpose, token_hash, expires_at)
        VALUES ($1, 'password_reset', $2, now() + interval '1 hour')",
        email,
        token_hash
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn reset_password(
    pool: &PgPool,
    redis: &Redis,
    token_hash: &str,
    password_hash: &str,
) -> Result<(), PasswordResetError> {
    let mut transaction = pool.begin().await?;
    let user_id = sqlx::query_scalar!(
        "SELECT users.id FROM verifications
        JOIN users ON users.email = verifications.email
        WHERE verifications.token_hash = $1 AND verifications.purpose = 'password_reset'
            AND verifications.consumed_at IS NULL AND verifications.expires_at > now()
        FOR UPDATE",
        token_hash
    )
    .fetch_optional(&mut *transaction)
    .await?;
    let Some(user_id) = user_id else {
        return Err(PasswordResetError::InvalidToken);
    };

    sqlx::query!(
        "INSERT INTO accounts (user_id, provider, password_hash) VALUES ($1, 'credential', $2)
        ON CONFLICT (user_id, provider)
        DO UPDATE SET password_hash = EXCLUDED.password_hash, updated_at = now()",
        user_id,
        password_hash
    )
    .execute(&mut *transaction)
    .await?;

    sqlx::query!(
        "UPDATE verifications SET consumed_at = now()
        WHERE purpose = 'password_reset' AND consumed_at IS NULL
            AND email = (SELECT email FROM users WHERE id = $1)",
        user_id
    )
    .execute(&mut *transaction)
    .await?;

    let revoked = sqlx::query_scalar!(
        "DELETE FROM sessions WHERE user_id = $1 RETURNING id",
        user_id
    )
    .fetch_all(&mut *transaction)
    .await?;

    access::revoke(redis, &revoked).await?;
    transaction.commit().await?;
    Ok(())
}
