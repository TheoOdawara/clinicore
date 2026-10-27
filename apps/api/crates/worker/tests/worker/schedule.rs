use std::time::{Duration, Instant};

use clinicore_worker::schedule;
use jiff::Timestamp;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

use crate::support::{capture_log, count};

#[sqlx::test(migrations = "../../migrations")]
async fn the_schedule_purges_at_three_in_the_morning_until_it_is_stopped(pool: PgPool) {
    sqlx::query(
        "INSERT INTO verifications (email, purpose, token_hash, expires_at)
        VALUES ('ana@example.com', 'email_verification', 'expired', now() - interval '1 second')",
    )
    .execute(&pool)
    .await
    .expect("an expired verification");
    let (log, _guard) = capture_log();
    let almost_three: Timestamp = "2026-10-20T02:59:59.8-03:00"
        .parse()
        .expect("a valid timestamp");
    let started = Instant::now();
    let clock = move || {
        almost_three
            .checked_add(started.elapsed())
            .expect("a timestamp in range")
    };
    let stop = CancellationToken::new();

    let running = tokio::spawn(schedule::run(pool.clone(), stop.clone(), clock));
    let deadline = Instant::now() + Duration::from_secs(5);
    while count(&pool, "SELECT count(*) FROM verifications").await > 0 {
        assert!(Instant::now() < deadline, "the purge never ran");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(5), running)
        .await
        .expect("a stopped schedule")
        .expect("a schedule that did not panic");

    assert!(
        log.lines().iter().any(|line| {
            line["message"] == "purged the expired authentication records"
                && line["verifications"] == 1
        }),
        "{:?}",
        log.lines()
    );
}
