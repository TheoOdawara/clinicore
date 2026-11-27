use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tokio::sync::watch;

use crate::support::capture_log;

#[tokio::test(start_paused = true)]
async fn the_schedule_purges_when_the_hour_comes_and_stops_on_the_signal() {
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy("postgresql://clinicore@127.0.0.1:1/clinicore")
        .expect("the url parses");
    let (log, _guard) = capture_log();
    let (stop, stopped) = watch::channel(false);
    let schedule = tokio::spawn(clinicore_worker::schedule::run(pool, stopped));

    tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
    let text = log.text();
    assert!(
        text.contains("could not purge the expired authentication records"),
        "{text}"
    );
    stop.send_replace(true);

    tokio::time::timeout(Duration::from_secs(5), schedule)
        .await
        .expect("the schedule stops")
        .expect("the schedule does not panic");
}
