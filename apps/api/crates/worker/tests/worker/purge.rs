use sqlx::PgPool;

use crate::support::capture_log;

async fn seed(pool: &PgPool) {
    for sql in [
        "INSERT INTO users (id, name, email) VALUES
             ('00000000-0000-0000-0000-000000000001', 'Ana', 'ana@exemplo.com')",
        "INSERT INTO sessions (user_id, client, refresh_token_hash, expires_at, created_at) VALUES
             ('00000000-0000-0000-0000-000000000001', 'web', '\\x01', now() - interval '1 second', now() - interval '1 day'),
             ('00000000-0000-0000-0000-000000000001', 'web', '\\x02', now() + interval '1 day', now() - interval '31 days'),
             ('00000000-0000-0000-0000-000000000001', 'mobile', '\\x03', now() + interval '1 day', now())",
        "INSERT INTO verifications (email, purpose, token_hash, expires_at) VALUES
             ('ana@exemplo.com', 'email_verification', '\\x01', now() - interval '1 second'),
             ('ana@exemplo.com', 'email_verification', '\\x02', now() + interval '1 hour')",
        "INSERT INTO email_dispatches (email, kind, created_at) VALUES
             ('ana@exemplo.com', 'email_verification', now() - interval '25 hours'),
             ('ana@exemplo.com', 'email_verification', now() - interval '23 hours')",
        "INSERT INTO email_outbox (email, subject, html_body, text_body, expires_at) VALUES
             ('ana@exemplo.com', 'Old', '<p>x</p>', 'x', now() - interval '1 second'),
             ('ana@exemplo.com', 'New', '<p>x</p>', 'x', now() + interval '1 hour')",
    ] {
        sqlx::query(sql).execute(pool).await.expect("the seed is written");
    }
}

async fn remaining(pool: &PgPool) -> [i64; 4] {
    let mut counts = [0; 4];
    for (index, sql) in [
        "SELECT count(*) FROM sessions",
        "SELECT count(*) FROM verifications",
        "SELECT count(*) FROM email_dispatches",
        "SELECT count(*) FROM email_outbox",
    ]
    .into_iter()
    .enumerate()
    {
        counts[index] = sqlx::query_scalar(sql)
            .fetch_one(pool)
            .await
            .expect("the table is readable");
    }
    counts
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_purge_deletes_what_expired_and_a_second_run_deletes_nothing(pool: PgPool) {
    seed(&pool).await;
    let (log, _guard) = capture_log();

    clinicore_worker::purge::run(&pool).await;

    assert_eq!(remaining(&pool).await, [1, 1, 1, 1]);
    let first = log.text();
    assert!(
        first.contains("purged the expired authentication records"),
        "{first}"
    );
    assert!(
        first.contains(r#""sessions":2,"verifications":1,"email_dispatches":1,"email_outbox":1"#),
        "{first}"
    );

    clinicore_worker::purge::run(&pool).await;

    assert_eq!(remaining(&pool).await, [1, 1, 1, 1]);
    let second = log.text()[first.len()..].to_owned();
    assert!(
        second.contains(r#""sessions":0,"verifications":0,"email_dispatches":0,"email_outbox":0"#),
        "{second}"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_second_instance_purging_again_deletes_nothing(pool: PgPool) {
    seed(&pool).await;

    clinicore_worker::purge::run(&pool).await;
    clinicore_worker::purge::run(&pool).await;

    assert_eq!(remaining(&pool).await, [1, 1, 1, 1]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_purge_that_fails_is_logged(pool: PgPool) {
    pool.close().await;
    let (log, _guard) = capture_log();

    clinicore_worker::purge::run(&pool).await;

    let text = log.text();
    assert!(text.contains(r#""level":"ERROR""#), "{text}");
    assert!(
        text.contains("could not purge the expired authentication records"),
        "{text}"
    );
}
