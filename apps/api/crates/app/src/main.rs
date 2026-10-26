use std::io;
use std::net::SocketAddr;
use std::process::ExitCode;

use clinicore_app::{AppState, GoogleClient, GoogleEndpoints};
use clinicore_core::config::Config;
use clinicore_core::redis::Redis;
use clinicore_core::{db, shutdown};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(rejection) => {
            eprintln!("{rejection}");
            return ExitCode::FAILURE;
        }
    };

    tracing::subscriber::set_global_default(clinicore_core::telemetry::subscriber(
        config.app_env,
        config.log_level,
        io::stdout,
    ))
    .expect("the only global subscriber");

    let state = AppState::new(
        &config,
        db::connect_lazy(&config.database_url),
        Redis::connect_lazy(&config),
        GoogleClient::new(&config, GoogleEndpoints::production()),
    );

    let listener = match TcpListener::bind(("::", config.port)).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(error = %error, port = config.port, "could not bind the port");
            return ExitCode::FAILURE;
        }
    };

    let app = clinicore_app::app(&config, state);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown::requested())
    .await
    .expect("axum::serve never returns an error");
    ExitCode::SUCCESS
}
