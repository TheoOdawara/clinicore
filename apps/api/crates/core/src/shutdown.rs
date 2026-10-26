use tokio::signal;

pub async fn requested() {
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("the runtime has a signal driver and SIGTERM can be handled")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        interrupted = signal::ctrl_c() => {
            interrupted.expect("the runtime has a signal driver and SIGINT can be handled");
        }
        () = terminate => {}
    }
}
