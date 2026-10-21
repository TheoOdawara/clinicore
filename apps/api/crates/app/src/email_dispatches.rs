use std::time::Duration;

use sqlx::{PgConnection, PgPool};

pub const DISPATCH_WINDOW: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "email_dispatch_kind", rename_all = "snake_case")]
pub enum EmailDispatchKind {
    EmailVerification,
    PasswordReset,
}

pub async fn claim_dispatch(
    connection: &mut PgConnection,
    email: &str,
    kind: EmailDispatchKind,
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
        WHERE email = $1 AND kind = $2 AND created_at > now() - make_interval(secs => $3)"#,
        email,
        kind as EmailDispatchKind,
        DISPATCH_WINDOW.as_secs_f64()
    )
    .fetch_one(&mut *connection)
    .await?;

    if window.total >= 5 || window.recent {
        return Ok(false);
    }

    sqlx::query!(
        "INSERT INTO email_dispatches (email, kind) VALUES ($1, $2)",
        email,
        kind as EmailDispatchKind
    )
    .execute(&mut *connection)
    .await?;
    Ok(true)
}

pub async fn reserve_dispatch(
    pool: &PgPool,
    email: &str,
    kind: EmailDispatchKind,
) -> Result<bool, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let claimed = claim_dispatch(&mut transaction, email, kind).await?;
    transaction.commit().await?;
    Ok(claimed)
}
