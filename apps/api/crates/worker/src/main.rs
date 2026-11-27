use std::process::ExitCode;

use clinicore_core::config::WorkerConfig;
use clinicore_core::{db, shutdown, telemetry};
use clinicore_worker::mail::Mailer;
use sqlx::migrate::Migrator;
use tokio::sync::watch;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[tokio::main]
async fn main() -> ExitCode {
    let config = match WorkerConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    telemetry::init(config.app_env, config.log_level)
        .expect("the tracing subscriber is installed once, at boot");

    let db = tokio::select! {
        result = db::connect_and_migrate(&config.database_url, &MIGRATOR) => match result {
            Ok(db) => db,
            Err(error) => {
                tracing::error!(%error, "boot failed");
                return ExitCode::FAILURE;
            }
        },
        () = shutdown::signal() => return ExitCode::SUCCESS,
    };

    let mailer = match Mailer::new(&config) {
        Ok(mailer) => mailer,
        Err(error) => {
            tracing::error!(%error, "boot failed");
            return ExitCode::FAILURE;
        }
    };

    let (stop, stopped) = watch::channel(false);
    tokio::spawn(async move {
        shutdown::signal().await;
        stop.send_replace(true);
    });
    tracing::info!("worker started");
    clinicore_worker::run(db, mailer, stopped).await;
    ExitCode::SUCCESS
}
