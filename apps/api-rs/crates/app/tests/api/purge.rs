use clinicore_app::purge::{Purged, purge_expired};
use sqlx::PgPool;
use uuid::Uuid;

use crate::support::count;

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

    let purged = purge_expired(&pool).await.expect("a purge");

    assert_eq!(
        purged,
        Purged {
            sessions: 2,
            verifications: 1,
            email_dispatches: 1
        }
    );
    let survivors = [
        "SELECT count(*) FROM sessions WHERE refresh_token_hash = 'live'",
        "SELECT count(*) FROM verifications WHERE token_hash = 'live'",
        "SELECT count(*) FROM email_dispatches WHERE created_at > now() - interval '24 hours'",
    ];
    for survivor in survivors {
        assert_eq!(count(&pool, survivor).await, 1, "{survivor}");
    }
    assert_eq!(
        purge_expired(&pool).await.expect("a second purge"),
        Purged {
            sessions: 0,
            verifications: 0,
            email_dispatches: 0
        }
    );
}
