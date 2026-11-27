use std::time::Duration;

use clinicore_core::db::interval;
use clinicore_core::email::RenderedEmail;
use clinicore_core::lifetimes::DISPATCH_WINDOW;
use sqlx::PgConnection;

use crate::credentials::email::Email;

const DISPATCH_SPACING: Duration = Duration::from_secs(60);
const DISPATCHES_PER_WINDOW: i64 = 5;

#[derive(Clone, Copy, Debug, sqlx::Type)]
#[sqlx(type_name = "email_dispatch_kind", rename_all = "snake_case")]
pub enum DispatchKind {
    EmailVerification,
}

pub async fn reserve(
    connection: &mut PgConnection,
    email: &Email,
    kind: DispatchKind,
) -> Result<bool, sqlx::Error> {
    sqlx::query!(
        r#"SELECT true AS "locked!" FROM pg_advisory_xact_lock(hashtextextended($1, 0))"#,
        email.as_str()
    )
    .fetch_one(&mut *connection)
    .await?;
    let recent = sqlx::query!(
        r#"
        SELECT
            count(*) AS "in_window!",
            count(*) FILTER (WHERE created_at > now() - $3::interval) AS "in_spacing!"
        FROM email_dispatches
        WHERE email = $1 AND kind = $2 AND created_at > now() - $4::interval
        "#,
        email.as_str(),
        kind as DispatchKind,
        interval(DISPATCH_SPACING),
        interval(DISPATCH_WINDOW),
    )
    .fetch_one(&mut *connection)
    .await?;
    if recent.in_spacing > 0 || recent.in_window >= DISPATCHES_PER_WINDOW {
        return Ok(false);
    }
    sqlx::query!(
        "INSERT INTO email_dispatches (email, kind) VALUES ($1, $2)",
        email.as_str(),
        kind as DispatchKind,
    )
    .execute(&mut *connection)
    .await?;
    Ok(true)
}

pub async fn enqueue(
    connection: &mut PgConnection,
    email: &Email,
    rendered: &RenderedEmail,
    lifetime: Duration,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO email_outbox (email, subject, html_body, text_body, expires_at)
        VALUES ($1, $2, $3, $4, now() + $5::interval)
        "#,
        email.as_str(),
        rendered.subject,
        rendered.html,
        rendered.text,
        interval(lifetime),
    )
    .execute(connection)
    .await?;
    Ok(())
}
