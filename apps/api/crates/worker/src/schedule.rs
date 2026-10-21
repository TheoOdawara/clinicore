use std::sync::LazyLock;

use croner::Cron;
use jiff::Zoned;
use sqlx::PgPool;

use crate::purge;

pub const TIME_ZONE: &str = "America/Sao_Paulo";

pub fn next_run(after: &Zoned) -> Zoned {
    static DAILY_AT_THREE: LazyLock<Cron> =
        LazyLock::new(|| "0 3 * * *".parse().expect("a valid cron expression"));
    DAILY_AT_THREE
        .find_next_occurrence(after, false)
        .expect("a daily schedule always has a next run")
}

pub async fn run(pool: PgPool) {
    loop {
        let now = Zoned::now()
            .in_tz(TIME_ZONE)
            .expect("the bundled time zone database has America/Sao_Paulo");
        let next = next_run(&now);
        tokio::time::sleep(now.duration_until(&next).unsigned_abs()).await;
        purge::purge_and_log(&pool).await;
    }
}
