use std::io;
use std::process::ExitCode;

use api_app::config::Config;
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

    let listener = match TcpListener::bind(("::", config.port)).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(error = %error, port = config.port, "could not bind the port");
            return ExitCode::FAILURE;
        }
    };

    if let Err(error) = axum::serve(listener, api_http::app(&config))
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!(error = %error, "the server stopped");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
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
