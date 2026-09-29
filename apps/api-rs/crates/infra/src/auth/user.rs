use crate::auth::email_dispatch::claim_verification;
use crate::auth::verification::insert_email_verification;
use crate::db::Database;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignUpOutcome {
    CreatedWithToken,
    Created,
    Duplicate,
}

pub struct UserSummary {
    pub name: String,
    pub email_verified: bool,
}

pub async fn create_with_credential_account(
    database: &Database,
    name: &str,
    email: &str,
    password_hash: &str,
    token_hash: &str,
) -> Result<SignUpOutcome, sqlx::Error> {
    let mut transaction = database.pool.begin().await?;
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

    if !claim_verification(&mut transaction, email).await? {
        transaction.commit().await?;
        return Ok(SignUpOutcome::Created);
    }

    insert_email_verification(&mut transaction, email, token_hash).await?;
    transaction.commit().await?;
    Ok(SignUpOutcome::CreatedWithToken)
}

pub async fn find_by_email(
    database: &Database,
    email: &str,
) -> Result<Option<UserSummary>, sqlx::Error> {
    sqlx::query_as!(
        UserSummary,
        "SELECT name, email_verified FROM users WHERE email = $1",
        email
    )
    .fetch_optional(&database.pool)
    .await
}
