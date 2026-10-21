use std::io;
use std::process::ExitCode;

use clinicore_core::config::WorkerConfig;
use clinicore_core::{db, telemetry};
use clinicore_worker::schedule;

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

    schedule::run(pool).await;
    ExitCode::SUCCESS
}
