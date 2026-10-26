use std::time::{Duration, Instant};

use clinicore_worker::schedule::{self, TIME_ZONE, next_run};
use jiff::{Timestamp, Zoned};
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

use crate::support::{capture_log, count};

fn in_sao_paulo(timestamp: &str) -> Zoned {
    timestamp
        .parse::<jiff::Timestamp>()
        .expect("a valid timestamp")
        .in_tz(TIME_ZONE)
        .expect("the bundled America/Sao_Paulo")
}

#[test]
fn the_next_purge_is_always_the_next_three_in_the_morning_in_sao_paulo() {
    assert_eq!(
        next_run(&in_sao_paulo("2026-10-20T02:59:59-03:00")),
        in_sao_paulo("2026-10-20T03:00:00-03:00")
    );
    assert_eq!(
        next_run(&in_sao_paulo("2026-10-20T03:00:00-03:00")),
        in_sao_paulo("2026-10-21T03:00:00-03:00")
    );
}

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
