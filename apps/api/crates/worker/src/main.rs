use std::io;
use std::process::ExitCode;

use clinicore_core::config::WorkerConfig;
use clinicore_core::{db, shutdown, telemetry};
use clinicore_worker::mail::Mailer;
use clinicore_worker::{outbox, schedule};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match WorkerConfig::from_env() {
        Ok(config) => config,
        Err(rejection) => {
            eprintln!("{rejection}");
            return ExitCode::FAILURE;
        }
    };

    tracing::subscriber::set_global_default(telemetry::subscriber(
        config.app_env,
        config.log_level,
        io::stdout,
    ))
    .expect("the only global subscriber");

    let pool = match db::connect_lazy(&config.database_url) {
        Ok(pool) => pool,
        Err(error) => {
            tracing::error!(error = %error, "could not set up the database pool");
            return ExitCode::FAILURE;
        }
    };
    let mailer = match Mailer::smtp(&config) {
        Ok(mailer) => mailer,
        Err(error) => {
            tracing::error!(error = %error, "could not set up the mail transport");
            return ExitCode::FAILURE;
        }
    };

    let stop = CancellationToken::new();
    tokio::join!(
        async {
            shutdown::requested().await;
            stop.cancel();
        },
        outbox::run(pool.clone(), mailer, stop.clone()),
        schedule::run(pool, stop.clone()),
    );
    ExitCode::SUCCESS
}
