use tokio::signal;
use tracing::info;

#[cfg(unix)]
pub async fn wait_for_shutdown() {
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("failed to install SIGTERM handler");
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
        .expect("failed to install SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down...");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT, shutting down...");
        }
    }
}

#[cfg(not(unix))]
pub async fn wait_for_shutdown() {
    signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    info!("Received Ctrl+C, shutting down...");
}
