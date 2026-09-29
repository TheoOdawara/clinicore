use sqlx::{PgConnection, PgExecutor, PgPool};

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
