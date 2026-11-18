use std::path::Path;
use std::time::Duration;

use clinicore_core::db::{BootError, connect_and_migrate};
use sqlx::ConnectOptions;
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

async fn migrator(folder: &str) -> Migrator {
    Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join(folder))
        .await
        .expect("the migrations folder is readable")
}

#[sqlx::test(migrations = false)]
async fn the_real_migrations_apply_to_an_empty_database(
    _: PgPoolOptions,
    options: PgConnectOptions,
) {
    let url = options.to_url_lossy();

    let pool = connect_and_migrate(url.as_str(), &migrator("../../migrations").await)
        .await
        .expect("the database is reachable and the migrations apply");

    for table in [
        "_sqlx_migrations",
        "users",
        "accounts",
        "sessions",
        "verifications",
        "email_dispatches",
        "email_outbox",
    ] {
        let exists: bool = sqlx::query_scalar("select to_regclass($1) is not null")
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("the database answers");
        assert!(exists, "{table} exists");
    }
}

#[sqlx::test(migrations = false)]
async fn a_failing_migration_stops_the_boot(_: PgPoolOptions, options: PgConnectOptions) {
    let url = options.to_url_lossy();

    let result = connect_and_migrate(
        url.as_str(),
        &migrator("tests/fixtures/broken_migrations").await,
    )
    .await;

    assert!(matches!(result, Err(BootError::Migration(_))));
}

#[tokio::test]
async fn a_refused_login_stops_the_boot_without_retrying() {
    let url = std::env::var("DATABASE_URL").expect("the tests run with DATABASE_URL");
    let mut options: PgConnectOptions = url.parse().expect("DATABASE_URL is a valid url");
    options = options.username("clinicore_role_that_does_not_exist");

    let result = tokio::time::timeout(
        Duration::from_secs(10),
        connect_and_migrate(
            options.to_url_lossy().as_str(),
            &migrator("../../migrations").await,
        ),
    )
    .await
    .expect("a refused login returns before the timeout");

    assert!(matches!(result, Err(BootError::Database(_))));
}

#[tokio::test]
async fn an_unreachable_database_keeps_the_boot_waiting() {
    let migrator = migrator("../../migrations").await;
    let attempt = connect_and_migrate("postgresql://clinicore@127.0.0.1:1/clinicore", &migrator);

    let outcome = tokio::time::timeout(Duration::from_secs(7), attempt).await;

    assert!(
        outcome.is_err(),
        "the boot is still waiting for the database"
    );
}
