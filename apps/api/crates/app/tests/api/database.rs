use clinicore_core::db;
use sqlx::postgres::PgConnectOptions;

#[tokio::test]
async fn every_pooled_connection_caps_how_long_a_statement_runs_and_waits_for_a_lock() {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL, injected by infisical run");
    let pool = db::connect_lazy(
        &database_url
            .parse::<PgConnectOptions>()
            .expect("a valid DATABASE_URL"),
    );

    let (statement, lock): (String, String) = sqlx::query_as(
        "SELECT current_setting('statement_timeout'), current_setting('lock_timeout')",
    )
    .fetch_one(&pool)
    .await
    .expect("both settings");

    assert_eq!((statement.as_str(), lock.as_str()), ("10s", "3s"));
}
