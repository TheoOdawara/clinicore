use std::io;
use std::process::ExitCode;

use clinicore_core::config::WorkerConfig;
use clinicore_core::{db, shutdown, telemetry};
use clinicore_worker::mail::Mailer;
use clinicore_worker::{outbox, schedule};
use jiff::Timestamp;
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

    let pool = db::connect_lazy(&config.database_url);
    let mailer = Mailer::smtp(&config);

    let stop = CancellationToken::new();
    tokio::join!(
        async {
            shutdown::requested().await;
            stop.cancel();
        },
        outbox::run(pool.clone(), mailer, stop.clone()),
        schedule::run(pool, stop.clone(), Timestamp::now),
    );
    ExitCode::SUCCESS
}
