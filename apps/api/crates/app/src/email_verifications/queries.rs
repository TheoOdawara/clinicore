use std::time::Duration;

use sqlx::PgConnection;

use crate::credentials::email::Email;
use clinicore_core::db::interval;

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
