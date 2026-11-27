use std::time::Duration;

use clinicore_core::db::interval;
use clinicore_core::lifetimes::SESSION_ABSOLUTE_LIFETIME;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::credentials::email::Email;
use crate::http::client::Client;
use crate::sessions::SessionId;
use crate::users::UserId;

pub struct Credential {
    pub user_id: Uuid,
    pub name: String,
    pub email_verified: bool,
    pub password_hash: Option<String>,
}

pub struct NewSession<'a> {
    pub user_id: UserId,
    pub client: Client,
    pub refresh_token_hash: &'a [u8],
    pub lifetime: Duration,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub struct Locked {
    pub client: Client,
    pub refresh_token_hash: Vec<u8>,
    pub user_id: Uuid,
    pub usable: bool,
}

pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

pub async fn credential_of(
    pool: &PgPool,
    email: &Email,
) -> Result<Option<Credential>, sqlx::Error> {
    sqlx::query_as!(
        Credential,
        r#"
        SELECT users.id AS user_id, users.name, users.email_verified, accounts.password_hash
        FROM users
        JOIN accounts ON accounts.user_id = users.id AND accounts.provider = 'credential'
        WHERE users.email = $1
        "#,
        email.as_str(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn lock_user(connection: &mut PgConnection, user_id: UserId) -> Result<(), sqlx::Error> {
    sqlx::query!("SELECT id FROM users WHERE id = $1 FOR UPDATE", user_id.0)
        .fetch_one(connection)
        .await?;
    Ok(())
}

pub async fn insert(
    connection: &mut PgConnection,
    session: NewSession<'_>,
) -> Result<SessionId, sqlx::Error> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO sessions (user_id, client, refresh_token_hash, expires_at, ip_address, user_agent)
        VALUES ($1, $2, $3, now() + $4::interval, $5::text::inet, $6)
        RETURNING id
        "#,
        session.user_id.0,
        session.client as Client,
        session.refresh_token_hash,
        interval(session.lifetime),
        session.ip_address,
        session.user_agent,
    )
    .fetch_one(connection)
    .await?;
    Ok(SessionId(id))
}

pub async fn evict_beyond(
    connection: &mut PgConnection,
    user_id: UserId,
    opened: SessionId,
    kept: i64,
) -> Result<Vec<SessionId>, sqlx::Error> {
    let ids = sqlx::query_scalar!(
        r#"
        DELETE FROM sessions
        WHERE user_id = $1 AND id <> $2 AND id NOT IN (
            SELECT id FROM sessions
            WHERE user_id = $1 AND id <> $2
            ORDER BY created_at DESC, id DESC
            LIMIT $3
        )
        RETURNING id
        "#,
        user_id.0,
        opened.0,
        kept,
    )
    .fetch_all(connection)
    .await?;
    Ok(ids.into_iter().map(SessionId).collect())
}

pub async fn locked(
    connection: &mut PgConnection,
    session_id: SessionId,
) -> Result<Option<Locked>, sqlx::Error> {
    sqlx::query_as!(
        Locked,
        r#"
        SELECT
            client AS "client: Client",
            refresh_token_hash,
            user_id,
            (expires_at > now() AND created_at + $2::interval > now()) AS "usable!"
        FROM sessions
        WHERE id = $1
        FOR UPDATE
        "#,
        session_id.0,
        interval(SESSION_ABSOLUTE_LIFETIME),
    )
    .fetch_optional(connection)
    .await
}

pub async fn rotate(
    connection: &mut PgConnection,
    session_id: SessionId,
    refresh_token_hash: &[u8],
    lifetime: Duration,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE sessions
        SET refresh_token_hash = $2,
            expires_at = least(now() + $3::interval, created_at + $4::interval),
            updated_at = now()
        WHERE id = $1
        "#,
        session_id.0,
        refresh_token_hash,
        interval(lifetime),
        interval(SESSION_ABSOLUTE_LIFETIME),
    )
    .execute(connection)
    .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, session_id: SessionId) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM sessions WHERE id = $1", session_id.0)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn user_of(pool: &PgPool, user_id: UserId) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, name, email FROM users WHERE id = $1",
        user_id.0,
    )
    .fetch_optional(pool)
    .await
}
