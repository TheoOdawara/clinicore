use clinicore_core::redis::Redis;
use ipnet::IpNet;
use sqlx::{PgConnection, PgExecutor, PgPool};
use uuid::Uuid;

use super::error::AuthError;
use super::responses::SessionUser;
use super::session::{self, Device};
use crate::http::client::SessionClient;

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

pub async fn claim_dispatch(
    connection: &mut PgConnection,
    email: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query!(
        r#"SELECT true AS "locked!" FROM pg_advisory_xact_lock(hashtext('email_dispatches'), hashtext($1))"#,
        email
    )
    .fetch_one(&mut *connection)
    .await?;

    let window = sqlx::query!(
        r#"SELECT count(*) AS "total!",
            coalesce(max(created_at) > now() - interval '60 seconds', false) AS "recent!"
        FROM email_dispatches
        WHERE email = $1 AND kind = 'email_verification' AND created_at > now() - interval '24 hours'"#,
        email
    )
    .fetch_one(&mut *connection)
    .await?;

    if window.total >= 5 || window.recent {
        return Ok(false);
    }

    sqlx::query!(
        "INSERT INTO email_dispatches (email, kind) VALUES ($1, 'email_verification')",
        email
    )
    .execute(&mut *connection)
    .await?;
    Ok(true)
}

pub async fn reserve_dispatch(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let claimed = claim_dispatch(&mut transaction, email).await?;
    transaction.commit().await?;
    Ok(claimed)
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

pub async fn find_credential(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(SessionUser, Option<String>)>, sqlx::Error> {
    let found = sqlx::query!(
        "SELECT users.id, users.name, users.email, users.email_verified, users.image, accounts.password_hash
        FROM users
        LEFT JOIN accounts ON accounts.user_id = users.id AND accounts.provider = 'credential'
        WHERE users.email = $1",
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
) -> Result<Uuid, AuthError> {
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
        session::lifetime(client).as_secs_f64(),
        device.ip_address.map(IpNet::from),
        device.user_agent,
        client as SessionClient
    )
    .fetch_one(&mut *transaction)
    .await?;

    let evicted = sqlx::query_scalar!(
        "DELETE FROM sessions WHERE id IN (
            SELECT id FROM sessions WHERE user_id = $1
            ORDER BY created_at DESC, id DESC OFFSET 5
        )
        RETURNING id",
        user_id
    )
    .fetch_all(&mut *transaction)
    .await?;

    session::revoke(redis, &evicted).await?;
    transaction.commit().await?;
    Ok(session_id)
}

pub enum Rotation {
    Rotated {
        user_id: Uuid,
        client: SessionClient,
    },
    Invalid,
    Reused,
}

pub async fn rotate_session(
    pool: &PgPool,
    redis: &Redis,
    session_id: Uuid,
    presented_hash: &str,
    new_hash: &str,
) -> Result<Rotation, AuthError> {
    let mut transaction = pool.begin().await?;
    let found = sqlx::query!(
        r#"SELECT user_id, client AS "client: SessionClient",
            expires_at > now() AS "live!", refresh_token_hash = $2 AS "matches!"
        FROM sessions WHERE id = $1 FOR UPDATE"#,
        session_id,
        presented_hash
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(found) = found.filter(|found| found.live) else {
        return Ok(Rotation::Invalid);
    };

    if !found.matches {
        sqlx::query!("DELETE FROM sessions WHERE id = $1", session_id)
            .execute(&mut *transaction)
            .await?;
        session::revoke(redis, &[session_id]).await?;
        transaction.commit().await?;
        return Ok(Rotation::Reused);
    }

    sqlx::query!(
        "UPDATE sessions
        SET refresh_token_hash = $2, expires_at = now() + make_interval(secs => $3), updated_at = now()
        WHERE id = $1",
        session_id,
        new_hash,
        session::lifetime(found.client).as_secs_f64()
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(Rotation::Rotated {
        user_id: found.user_id,
        client: found.client,
    })
}

pub async fn close_session(
    pool: &PgPool,
    redis: &Redis,
    session_id: Uuid,
) -> Result<bool, AuthError> {
    let mut transaction = pool.begin().await?;
    let deleted = sqlx::query!("DELETE FROM sessions WHERE id = $1", session_id)
        .execute(&mut *transaction)
        .await?;
    if deleted.rows_affected() == 0 {
        return Ok(false);
    }
    session::revoke(redis, &[session_id]).await?;
    transaction.commit().await?;
    Ok(true)
}

pub enum Confirmation {
    Confirmed { user_id: Uuid },
    Invalid,
    Expired,
}

pub async fn consume_email_verification(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Confirmation, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let found = sqlx::query!(
        r#"SELECT email, expires_at > now() AS "live!"
        FROM verifications
        WHERE token_hash = $1 AND purpose = 'email_verification' AND consumed_at IS NULL
        FOR UPDATE"#,
        token_hash
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(found) = found else {
        return Ok(Confirmation::Invalid);
    };
    if !found.live {
        return Ok(Confirmation::Expired);
    }

    let user_id = sqlx::query_scalar!(
        "UPDATE users SET email_verified = true, updated_at = now() WHERE email = $1 RETURNING id",
        found.email
    )
    .fetch_optional(&mut *transaction)
    .await?;
    let Some(user_id) = user_id else {
        return Ok(Confirmation::Invalid);
    };

    sqlx::query!(
        "UPDATE verifications SET consumed_at = now()
        WHERE email = $1 AND purpose = 'email_verification' AND consumed_at IS NULL",
        found.email
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(Confirmation::Confirmed { user_id })
}
