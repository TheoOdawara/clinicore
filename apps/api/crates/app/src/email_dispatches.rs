use clinicore_core::lifetimes::{DISPATCH_COOLDOWN, DISPATCH_LIMIT, DISPATCH_WINDOW};
use sqlx::PgConnection;

#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "email_dispatch_kind", rename_all = "snake_case")]
pub enum EmailDispatchKind {
    EmailVerification,
    PasswordReset,
}

pub fn dispatch_limits_text() -> String {
    format!(
        "At most one link per address every {} seconds and {DISPATCH_LIMIT} every {} hours.",
        DISPATCH_COOLDOWN.as_secs(),
        DISPATCH_WINDOW.as_secs() / 3600
    )
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
            coalesce(max(created_at) > now() - make_interval(secs => $4), false) AS "recent!"
        FROM email_dispatches
        WHERE email = $1 AND kind = $2 AND created_at > now() - make_interval(secs => $3)"#,
        email,
        kind as EmailDispatchKind,
        DISPATCH_WINDOW.as_secs_f64(),
        DISPATCH_COOLDOWN.as_secs_f64()
    )
    .fetch_one(&mut *connection)
    .await?;

    if window.total >= DISPATCH_LIMIT || window.recent {
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
