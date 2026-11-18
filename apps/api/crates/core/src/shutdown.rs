use tokio::signal::unix::{SignalKind, signal as unix_signal};

pub async fn signal() {
    let mut terminate = unix_signal(SignalKind::terminate())
        .expect("the SIGTERM handler is registered once the tokio runtime is running");
    tokio::select! {
        _ = terminate.recv() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn sigterm_resolves_the_shutdown_signal() {
        let waiting = tokio::spawn(signal());
        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = Command::new("kill")
            .args(["-TERM", &std::process::id().to_string()])
            .status()
            .expect("kill is available");
        assert!(status.success());

        tokio::time::timeout(Duration::from_secs(5), waiting)
            .await
            .expect("the signal resolves before the timeout")
            .expect("the signal task does not panic");
    }
}
