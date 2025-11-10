mod config;
mod indexer;
mod o11y;
mod routes;

use std::sync::Arc;

use anyhow::Result;
use config::AppConfig;
use indexer::{IndexEvent, Indexer, IndexerConfig};
use routes::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(AppConfig::load()?);

    let _telemetry = o11y::TelemetryGuard::init(&config)?;
    let state = AppState::new(config.clone());
    let (indexer_handle, mut index_events) =
        Indexer::spawn(IndexerConfig::new(config.media_root.clone()));

    tokio::spawn(async move {
        while let Some(event) = index_events.recv().await {
            match event {
                IndexEvent::Snapshot {
                    files, duration, ..
                } => {
                    tracing::info!(
                        count = files.len(),
                        elapsed_ms = duration.as_millis(),
                        "filesystem scan complete"
                    );
                }
                IndexEvent::Error { message } => {
                    tracing::warn!(%message, "indexer error");
                }
            }
        }
    });

    let listener = tokio::net::TcpListener::bind(config.listen_addr).await?;
    tracing::info!(addr = %config.listen_addr, "HTTP server listening");

    axum::serve(listener, routes::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Ensure the indexer task stops when the server exits.
    indexer_handle.abort();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install signal handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
