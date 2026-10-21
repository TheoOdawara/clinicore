use std::time::Duration;

use sqlx::PgPool;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::mail::Mailer;

const MAX_ATTEMPTS: i32 = 5;

pub struct Claimed {
    pub id: Uuid,
    pub email: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
    pub attempts: i32,
}

pub async fn run(pool: PgPool, mailer: Mailer, shutdown: CancellationToken) {
    while !shutdown.is_cancelled() {
        match claim(&pool).await {
            Ok(claimed) => {
                for mail in claimed {
                    if shutdown.is_cancelled() {
                        break;
                    }
                    if let Err(error) = deliver(&pool, &mailer, mail).await {
                        tracing::error!(error = %error, "could not record the mail delivery");
                    }
                }
            }
            Err(error) => tracing::error!(error = %error, "could not claim the pending mail"),
        }
        tokio::select! {
            _ = shutdown.cancelled() => {}
            _ = tokio::time::sleep(Duration::from_secs(5)) => {}
        }
    }
}

pub async fn claim(pool: &PgPool) -> Result<Vec<Claimed>, sqlx::Error> {
    sqlx::query_as!(
        Claimed,
        r#"UPDATE email_outbox
        SET attempts = attempts + 1, next_attempt_at = now() + interval '2 minutes'
        WHERE id IN (
            SELECT id FROM email_outbox
            WHERE status = 'pending' AND next_attempt_at <= now() AND expires_at > now()
                AND attempts < $1
            ORDER BY next_attempt_at
            LIMIT 10
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id, email, subject, html_body AS "html_body!", text_body AS "text_body!", attempts"#,
        MAX_ATTEMPTS
    )
    .fetch_all(pool)
    .await
}

pub async fn deliver(pool: &PgPool, mailer: &Mailer, mail: Claimed) -> Result<(), sqlx::Error> {
    let Err(reason) = mailer
        .deliver(&mail.email, mail.subject, mail.html_body, mail.text_body)
        .await
    else {
        sqlx::query!("DELETE FROM email_outbox WHERE id = $1", mail.id)
            .execute(pool)
            .await?;
        return Ok(());
    };
    if mail.attempts < MAX_ATTEMPTS {
        sqlx::query!(
            "UPDATE email_outbox SET next_attempt_at = now() + make_interval(mins => 1 << (attempts - 1))
            WHERE id = $1",
            mail.id
        )
        .execute(pool)
        .await?;
        tracing::warn!(
            email = %masked(&mail.email),
            attempt = mail.attempts,
            reason = %reason,
            "mail delivery failed, retrying"
        );
    } else {
        sqlx::query!(
            "UPDATE email_outbox SET status = 'failed', html_body = NULL, text_body = NULL
            WHERE id = $1",
            mail.id
        )
        .execute(pool)
        .await?;
        tracing::error!(
            email = %masked(&mail.email),
            attempts = mail.attempts,
            reason = %reason,
            "mail delivery gave up"
        );
    }
    Ok(())
}

fn masked(address: &str) -> String {
    let Some((local, domain)) = address.split_once('@') else {
        return "***".to_string();
    };
    let first: String = local.chars().take(1).collect();
    format!("{first}***@{domain}")
}
