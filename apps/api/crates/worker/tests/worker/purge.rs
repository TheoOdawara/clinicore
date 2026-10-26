use clinicore_worker::purge::{Purged, purge_and_log, purge_expired};
use sqlx::PgPool;
use uuid::Uuid;

use crate::support::{capture_log, count};

#[sqlx::test(migrations = "../../migrations")]
async fn the_purge_deletes_only_what_expired_and_a_second_run_deletes_nothing(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (name, email) VALUES ('Ana Souza', 'ana@example.com') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("a user");
    sqlx::query(
        "INSERT INTO sessions (user_id, refresh_token_hash, expires_at, created_at) VALUES
        ($1, 'expired', now() - interval '1 second', now() - interval '2 days'),
        ($1, 'past the absolute lifetime', now() + interval '1 day', now() - interval '31 days'),
        ($1, 'live', now() + interval '1 day', now() - interval '29 days')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("the sessions");
    sqlx::query(
        "INSERT INTO verifications (email, purpose, token_hash, expires_at) VALUES
        ('ana@example.com', 'email_verification', 'expired', now() - interval '1 second'),
        ('ana@example.com', 'email_verification', 'live', now() + interval '1 hour')",
    )
    .execute(&pool)
    .await
    .expect("the verifications");
    sqlx::query(
        "INSERT INTO email_dispatches (email, kind, created_at) VALUES
        ('ana@example.com', 'email_verification', now() - interval '25 hours'),
        ('ana@example.com', 'email_verification', now() - interval '23 hours')",
    )
    .execute(&pool)
    .await
    .expect("the dispatches");
    sqlx::query(
        "INSERT INTO email_outbox (email, subject, html_body, text_body, expires_at) VALUES
        ('ana@example.com', 'expired', '<p>link</p>', 'link', now() - interval '1 second'),
        ('ana@example.com', 'live', '<p>link</p>', 'link', now() + interval '1 hour')",
    )
    .execute(&pool)
    .await
    .expect("the queued mail");

    let purged = purge_expired(&pool).await.expect("a purge");

    assert_eq!(
        purged,
        Purged {
            sessions: 2,
            verifications: 1,
            email_dispatches: 1,
            email_outbox: 1
        }
    );
    let survivors = [
        "SELECT count(*) FROM sessions WHERE refresh_token_hash = 'live'",
        "SELECT count(*) FROM verifications WHERE token_hash = 'live'",
        "SELECT count(*) FROM email_dispatches WHERE created_at > now() - interval '24 hours'",
        "SELECT count(*) FROM email_outbox WHERE subject = 'live'",
    ];
    for survivor in survivors {
        assert_eq!(count(&pool, survivor).await, 1, "{survivor}");
    }
    assert_eq!(
        purge_expired(&pool).await.expect("a second purge"),
        Purged {
            sessions: 0,
            verifications: 0,
            email_dispatches: 0,
            email_outbox: 0
        }
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_table_that_cannot_be_purged_stops_the_purge_and_is_logged(pool: PgPool) {
    let tables = [
        (
            "ALTER TABLE sessions RENAME TO sessions_away",
            "ALTER TABLE sessions_away RENAME TO sessions",
        ),
        (
            "ALTER TABLE verifications RENAME TO verifications_away",
            "ALTER TABLE verifications_away RENAME TO verifications",
        ),
        (
            "ALTER TABLE email_dispatches RENAME TO email_dispatches_away",
            "ALTER TABLE email_dispatches_away RENAME TO email_dispatches",
        ),
        (
            "ALTER TABLE email_outbox RENAME TO email_outbox_away",
            "ALTER TABLE email_outbox_away RENAME TO email_outbox",
        ),
    ];
    for (away, back) in tables {
        sqlx::query(away)
            .execute(&pool)
            .await
            .expect("a missing table");
        assert!(purge_expired(&pool).await.is_err(), "{away}");
        sqlx::query(back)
            .execute(&pool)
            .await
            .expect("the table back");
    }

    let (log, _guard) = capture_log();
    let (away, back) = tables[0];
    sqlx::query(away)
        .execute(&pool)
        .await
        .expect("a missing table");
    purge_and_log(&pool).await;
    sqlx::query(back)
        .execute(&pool)
        .await
        .expect("the table back");

    assert!(
        log.lines().iter().any(|line| {
            line["level"] == "ERROR"
                && line["message"] == "could not purge the expired authentication records"
        }),
        "{:?}",
        log.lines()
    );
}
