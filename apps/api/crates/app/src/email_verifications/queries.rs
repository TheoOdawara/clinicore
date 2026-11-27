use std::time::Duration;

use clinicore_core::db::interval;
use sqlx::PgConnection;

use crate::credentials::email::Email;

pub struct TokenState {
    pub consumed: bool,
    pub expired: bool,
}

pub async fn insert(
    connection: &mut PgConnection,
    email: &Email,
    token_hash: &[u8],
    lifetime: Duration,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO verifications (email, purpose, token_hash, expires_at)
        VALUES ($1, 'email_verification', $2, now() + $3::interval)
        "#,
        email.as_str(),
        token_hash,
        interval(lifetime),
    )
    .execute(connection)
    .await?;
    Ok(())
}

pub async fn unverified_name(
    connection: &mut PgConnection,
    email: &Email,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT name FROM users WHERE email = $1 AND NOT email_verified",
        email.as_str(),
    )
    .fetch_optional(connection)
    .await
}

pub async fn email_of(
    connection: &mut PgConnection,
    token_hash: &[u8],
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT email FROM verifications WHERE token_hash = $1 AND purpose = 'email_verification'",
        token_hash,
    )
    .fetch_optional(connection)
    .await
}

pub async fn lock_address(connection: &mut PgConnection, email: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"SELECT true AS "locked!" FROM pg_advisory_xact_lock(hashtextextended($1, 0))"#,
        email,
    )
    .fetch_one(connection)
    .await?;
    Ok(())
}

pub async fn token_state(
    connection: &mut PgConnection,
    token_hash: &[u8],
) -> Result<TokenState, sqlx::Error> {
    sqlx::query_as!(
        TokenState,
        r#"
        SELECT consumed_at IS NOT NULL AS "consumed!", expires_at <= now() AS "expired!"
        FROM verifications
        WHERE token_hash = $1
        "#,
        token_hash,
    )
    .fetch_one(connection)
    .await
}

pub async fn verify(connection: &mut PgConnection, email: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET email_verified = true, updated_at = now() WHERE email = $1",
        email,
    )
    .execute(&mut *connection)
    .await?;
    sqlx::query!(
        r#"
        UPDATE verifications SET consumed_at = now()
        WHERE email = $1 AND purpose = 'email_verification' AND consumed_at IS NULL
        "#,
        email,
    )
    .execute(connection)
    .await?;
    Ok(())
}
