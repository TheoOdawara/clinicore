use std::time::Duration;

use clinicore_core::db::interval;
use sqlx::PgPool;
use tokio::sync::watch;
use uuid::Uuid;

use crate::mail::{MailError, Mailer, Outgoing};

const CYCLE_INTERVAL: Duration = Duration::from_secs(5);
const CLAIM_LIMIT: i64 = 10;
const CLAIM_LEASE: Duration = Duration::from_secs(2 * 60);
const MAX_ATTEMPTS: i32 = 5;

pub struct Claimed {
    pub id: Uuid,
    pub email: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
    pub attempts: i32,
}

pub async fn run(pool: PgPool, mailer: Mailer, mut stop: watch::Receiver<bool>) {
    let mut ticker = tokio::time::interval(CYCLE_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = stop.changed() => return,
        }
        cycle(&pool, &mailer, &stop).await;
    }
}

pub async fn cycle(pool: &PgPool, mailer: &Mailer, stop: &watch::Receiver<bool>) {
    let claimed = match claim(pool).await {
        Ok(claimed) => claimed,
        Err(error) => {
            tracing::error!(%error, "could not claim the pending mail");
            return;
        }
    };
    for mail in claimed {
        if *stop.borrow() {
            return;
        }
        let outcome = mailer
            .send(&Outgoing {
                to: &mail.email,
                subject: &mail.subject,
                html: &mail.html_body,
                text: &mail.text_body,
            })
            .await;
        if let Err(error) = record(pool, &mail, outcome).await {
            tracing::error!(%error, "could not record the mail result");
        }
    }
}

pub async fn claim(pool: &PgPool) -> Result<Vec<Claimed>, sqlx::Error> {
    sqlx::query_as!(
        Claimed,
        r#"
        UPDATE email_outbox
        SET attempts = attempts + 1, next_attempt_at = now() + $1::interval
        FROM (
            SELECT id FROM email_outbox
            WHERE status = 'pending'
              AND attempts < $2
              AND next_attempt_at <= now()
              AND expires_at > now()
            ORDER BY next_attempt_at
            LIMIT $3
            FOR UPDATE SKIP LOCKED
        ) AS claimed
        WHERE email_outbox.id = claimed.id
        RETURNING email_outbox.id, email, subject,
            html_body AS "html_body!", text_body AS "text_body!", attempts
        "#,
        interval(CLAIM_LEASE),
        MAX_ATTEMPTS,
        CLAIM_LIMIT,
    )
    .fetch_all(pool)
    .await
}

async fn record(
    pool: &PgPool,
    mail: &Claimed,
    outcome: Result<(), MailError>,
) -> Result<(), sqlx::Error> {
    let Err(reason) = outcome else {
        sqlx::query!("DELETE FROM email_outbox WHERE id = $1", mail.id)
            .execute(pool)
            .await?;
        return Ok(());
    };
    let email = mask(&mail.email);
    if mail.attempts < MAX_ATTEMPTS {
        sqlx::query!(
            "UPDATE email_outbox SET next_attempt_at = now() + $2::interval WHERE id = $1",
            mail.id,
            interval(backoff(mail.attempts)),
        )
        .execute(pool)
        .await?;
        tracing::warn!(%email, attempt = mail.attempts, %reason, "mail delivery failed, retrying");
    } else {
        sqlx::query!(
            r#"
            UPDATE email_outbox SET status = 'failed', html_body = NULL, text_body = NULL
            WHERE id = $1
            "#,
            mail.id,
        )
        .execute(pool)
        .await?;
        tracing::error!(%email, attempts = mail.attempts, %reason, "mail delivery gave up");
    }
    Ok(())
}

fn backoff(attempts: i32) -> Duration {
    Duration::from_secs(60 << (attempts - 1))
}

fn mask(email: &str) -> String {
    let (local, domain) = email
        .rsplit_once('@')
        .expect("a queued address was validated with its @");
    let first: String = local.chars().take(1).collect();
    format!("{first}***@{domain}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failure_waits_1_2_4_and_8_minutes() {
        let waits: Vec<u64> = (1..=4)
            .map(|attempts| backoff(attempts).as_secs())
            .collect();

        assert_eq!(waits, [60, 120, 240, 480]);
    }

    #[test]
    fn the_address_is_masked_after_its_first_character() {
        assert_eq!(mask("ana@exemplo.com"), "a***@exemplo.com");
        assert_eq!(mask("ção@exemplo.com"), "ç***@exemplo.com");
    }
}
