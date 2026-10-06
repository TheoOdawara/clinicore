use sqlx::PgPool;

use crate::email_dispatches::{EmailDispatchKind, claim_dispatch};
use crate::email_verifications::queries::create_verification;

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
