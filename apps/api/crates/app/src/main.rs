use std::net::{Ipv4Addr, SocketAddr};
use std::process::ExitCode;
use std::sync::Arc;

use clinicore_app::credentials::password;
use clinicore_app::{AppState, router};
use clinicore_core::config::ApiConfig;
use clinicore_core::{db, redis, shutdown, telemetry};
use sqlx::migrate::Migrator;
use tokio::net::TcpListener;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[tokio::main]
async fn main() -> ExitCode {
    let config = match ApiConfig::from_env() {
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

    let redis = match redis::connect(&config.redis_url) {
        Ok(redis) => redis,
        Err(error) => {
            tracing::error!(%error, "boot failed");
            return ExitCode::FAILURE;
        }
    };

    password::prepare();

    let listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, config.port)).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, port = config.port, "the port is unavailable");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(port = config.port, "listening");

    let state = AppState {
        db,
        redis,
        config: Arc::new(config),
    };
    let served = axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown::signal())
    .await;
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "the server stopped");
            ExitCode::FAILURE
        }
    }
}
