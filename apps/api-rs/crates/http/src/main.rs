use std::io;
use std::net::SocketAddr;
use std::process::ExitCode;

use api_app::config::Config;
use api_app::{Database, Mailer, Redis, Services};
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(rejection) => {
            eprintln!("{rejection}");
            return ExitCode::FAILURE;
        }
    };

    tracing::subscriber::set_global_default(api_http::telemetry::subscriber(&config, io::stdout))
        .expect("the only global subscriber");

    let services = match connect(&config) {
        Ok(services) => services,
        Err(error) => {
            tracing::error!(error = %error, "could not set up the clients");
            return ExitCode::FAILURE;
        }
    };

    let listener = match TcpListener::bind(("::", config.port)).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(error = %error, port = config.port, "could not bind the port");
            return ExitCode::FAILURE;
        }
    };

    let app = api_http::app(&config, services);
    if let Err(error) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    {
        tracing::error!(error = %error, "the server stopped");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn connect(config: &Config) -> Result<Services, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Services::new(
        config,
        Database::connect_lazy(config)?,
        Redis::connect_lazy(config)?,
        Mailer::smtp(config)?,
    ))
}

async fn shutdown_signal() {
    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                terminate.recv().await;
            }
            Err(error) => {
                tracing::error!(error = %error, "could not listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = signal::ctrl_c() => {}
        _ = terminate => {}
    }
}
