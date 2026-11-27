use croner::Cron;
use jiff::ZonedDateTime;
use sqlx::PgPool;
use tokio::sync::watch;

use crate::purge;

const PURGE_PATTERN: &str = "0 3 * * *";
const PURGE_TIME_ZONE: &str = "America/Sao_Paulo";

pub async fn run(pool: PgPool, mut stop: watch::Receiver<bool>) {
    loop {
        let now = ZonedDateTime::now();
        let wait = now.duration_until(&next_purge(&now)).unsigned_abs();
        tokio::select! {
            () = tokio::time::sleep(wait) => {}
            _ = stop.changed() => return,
        }
        purge::run(&pool).await;
    }
}

pub fn next_purge(after: &ZonedDateTime) -> ZonedDateTime {
    let cron: Cron = PURGE_PATTERN
        .parse()
        .expect("the purge pattern is a valid cron expression");
    let local = after
        .in_tz(PURGE_TIME_ZONE)
        .expect("the time zone database is bundled into the binary");
    cron.find_next_occurrence(&local, false)
        .expect("a daily pattern always has a next occurrence")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> ZonedDateTime {
        text.parse().expect("the instant parses")
    }

    #[test]
    fn the_next_purge_is_the_next_3_am_in_sao_paulo() {
        assert_eq!(
            next_purge(&at("2026-11-20T02:59:59-03:00[America/Sao_Paulo]")),
            at("2026-11-20T03:00:00-03:00[America/Sao_Paulo]")
        );
        assert_eq!(
            next_purge(&at("2026-11-20T03:00:00-03:00[America/Sao_Paulo]")),
            at("2026-11-21T03:00:00-03:00[America/Sao_Paulo]")
        );
        assert_eq!(
            next_purge(&at("2026-11-20T05:59:59+00:00[UTC]")),
            at("2026-11-20T03:00:00-03:00[America/Sao_Paulo]")
        );
    }
}
