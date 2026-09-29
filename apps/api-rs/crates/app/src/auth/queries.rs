use sqlx::{PgConnection, PgExecutor, PgPool};

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

    if !claim_dispatch(&mut transaction, email).await? {
        transaction.commit().await?;
        return Ok(SignUpOutcome::Created);
    }

    create_verification(&mut *transaction, email, token_hash).await?;
    transaction.commit().await?;
    Ok(SignUpOutcome::CreatedWithToken)
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

async fn claim_dispatch(connection: &mut PgConnection, email: &str) -> Result<bool, sqlx::Error> {
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
