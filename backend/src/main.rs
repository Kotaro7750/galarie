use std::sync::Arc;

use anyhow::Result;
use galarie_backend::{
    cache::CacheStore,
    config::AppConfig,
    indexer::{IndexEvent, Indexer, IndexerConfig},
    o11y,
    routes::{self, AppState},
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(AppConfig::load()?);
    let _telemetry = o11y::TelemetryGuard::init(&config)?;

    tracing::info!("starting Galarie backend with config {:?}", config);

    let cache_store = Arc::new(CacheStore::new(config.cache_dir.clone()));
    let media_root_for_cache = config.media_root.clone();
    let initial_snapshot =
        cache_store.load_or_rebuild(|| Indexer::scan_once(&media_root_for_cache))?;
    let snapshot_state = Arc::new(RwLock::new(initial_snapshot));

    let state = AppState::new(config.clone(), cache_store.clone(), snapshot_state.clone());
    let (indexer_handle, mut index_events) =
        Indexer::spawn(IndexerConfig::new(config.media_root.clone()));

    let cache_store_for_task = cache_store.clone();
    let snapshot_state_for_task = snapshot_state.clone();
    tokio::spawn(async move {
        while let Some(event) = index_events.recv().await {
            match event {
                IndexEvent::Snapshot {
                    files,
                    duration,
                    scanned_at,
                } => {
                    let elapsed_ms = duration.as_millis();
                    let file_count = files.len();

                    tracing::info!(
                        elapsed_ms,
                        file_count = file_count,
                        scanned_at = %scanned_at.to_rfc3339(),
                        "filesystem scan complete in {elapsed_ms} ms, found {file_count} files",
                    );

                    match cache_store_for_task.persist(files) {
                        Ok(snapshot) => {
                            *snapshot_state_for_task.write().await = snapshot.clone();
                            tracing::info!("filesystem scan persisted to cache");
                        }
                        Err(err) => {
                            tracing::error!(error = %err, "failed to persist cache snapshot");
                        }
                    }
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
