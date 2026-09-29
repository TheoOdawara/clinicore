use sqlx::PgConnection;

use crate::db::Database;

pub(crate) async fn insert_email_verification(
    connection: &mut PgConnection,
    identifier: &str,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO verifications (identifier, purpose, token_hash, expires_at)
        VALUES ($1, 'email_verification', $2, now() + interval '1 hour')",
        identifier,
        token_hash
    )
    .execute(connection)
    .await?;
    Ok(())
}

pub async fn create_email_verification(
    database: &Database,
    identifier: &str,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    let mut connection = database.pool.acquire().await?;
    insert_email_verification(&mut connection, identifier, token_hash).await
}
