use sqlx::PgConnection;

use crate::credentials::email::Email;
use crate::users::UserId;

pub async fn insert_user(
    connection: &mut PgConnection,
    name: &str,
    email: &Email,
) -> Result<Option<UserId>, sqlx::Error> {
    let id = sqlx::query_scalar!(
        "INSERT INTO users (name, email) VALUES ($1, $2) ON CONFLICT (email) DO NOTHING RETURNING id",
        name,
        email.as_str(),
    )
    .fetch_optional(connection)
    .await?;
    Ok(id.map(UserId))
}

pub async fn insert_credential_account(
    connection: &mut PgConnection,
    user_id: UserId,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO accounts (user_id, provider, password_hash) VALUES ($1, 'credential', $2)",
        user_id.0,
        password_hash,
    )
    .execute(connection)
    .await?;
    Ok(())
}
