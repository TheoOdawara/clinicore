use clinicore_worker::schedule::{TIME_ZONE, next_run};
use jiff::Zoned;

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
