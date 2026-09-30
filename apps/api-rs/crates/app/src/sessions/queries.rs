use clinicore_core::redis::Redis;
use ipnet::IpNet;
use sqlx::PgPool;
use uuid::Uuid;

use super::error::SessionError;
use super::extractors::Device;
use super::responses::SessionUser;
use super::tokens::{access, refresh};
use crate::http::client::SessionClient;

pub async fn find_credential(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(SessionUser, String)>, sqlx::Error> {
    let found = sqlx::query!(
        r#"SELECT users.id, users.name, users.email, users.email_verified, users.image,
            accounts.password_hash AS "password_hash!"
        FROM users
        JOIN accounts ON accounts.user_id = users.id AND accounts.provider = 'credential'
        WHERE users.email = $1"#,
        email
    )
    .fetch_optional(pool)
    .await?;
    Ok(found.map(|row| {
        let user = SessionUser {
            id: row.id,
            name: row.name,
            email: row.email,
            email_verified: row.email_verified,
            image: row.image,
        };
        (user, row.password_hash)
    }))
}

pub async fn find_session_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<SessionUser>, sqlx::Error> {
    sqlx::query_as!(
        SessionUser,
        "SELECT id, name, email, email_verified, image FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn open_session(
    pool: &PgPool,
    redis: &Redis,
    user_id: Uuid,
    refresh_hash: &str,
    client: SessionClient,
    device: Device,
) -> Result<Uuid, SessionError> {
    let mut transaction = pool.begin().await?;
    sqlx::query!("SELECT id FROM users WHERE id = $1 FOR UPDATE", user_id)
        .fetch_one(&mut *transaction)
        .await?;

    let session_id = sqlx::query_scalar!(
        "INSERT INTO sessions (user_id, refresh_token_hash, expires_at, ip_address, user_agent, client)
        VALUES ($1, $2, now() + make_interval(secs => $3), $4, $5, $6)
        RETURNING id",
        user_id,
        refresh_hash,
        refresh::lifetime(client).as_secs_f64(),
        device.ip_address.map(IpNet::from),
        device.user_agent,
        client as SessionClient
    )
    .fetch_one(&mut *transaction)
    .await?;

    let evicted = sqlx::query_scalar!(
        "DELETE FROM sessions WHERE id IN (
            SELECT id FROM sessions WHERE user_id = $1 AND id <> $2
            ORDER BY created_at DESC, id DESC OFFSET 4
        )
        RETURNING id",
        user_id,
        session_id
    )
    .fetch_all(&mut *transaction)
    .await?;

    access::revoke(redis, &evicted).await?;
    transaction.commit().await?;
    Ok(session_id)
}

pub async fn rotate_session(
    pool: &PgPool,
    redis: &Redis,
    session_id: Uuid,
    client: SessionClient,
    presented_hash: &str,
    new_hash: &str,
) -> Result<Uuid, SessionError> {
    let mut transaction = pool.begin().await?;
    let found = sqlx::query!(
        r#"SELECT user_id, client AS "client: SessionClient",
            expires_at > now() AND created_at > now() - make_interval(secs => $3) AS "live!",
            refresh_token_hash = $2 AS "matches!"
        FROM sessions WHERE id = $1 FOR UPDATE"#,
        session_id,
        presented_hash,
        refresh::ABSOLUTE_LIFETIME.as_secs_f64()
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(found) = found.filter(|found| found.live && found.client == client) else {
        return Err(SessionError::InvalidSession);
    };

    if !found.matches {
        transaction.rollback().await?;
        close_session(pool, redis, session_id).await?;
        return Err(SessionError::SessionReused);
    }

    sqlx::query!(
        "UPDATE sessions
        SET refresh_token_hash = $2, expires_at = now() + make_interval(secs => $3), updated_at = now()
        WHERE id = $1",
        session_id,
        new_hash,
        refresh::lifetime(client).as_secs_f64()
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(found.user_id)
}

pub async fn close_session(
    pool: &PgPool,
    redis: &Redis,
    session_id: Uuid,
) -> Result<(), SessionError> {
    access::revoke(redis, &[session_id]).await?;
    let deleted = sqlx::query!("DELETE FROM sessions WHERE id = $1", session_id)
        .execute(pool)
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(SessionError::InvalidSession);
    }
    Ok(())
}
