use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
    vec::Vec,
};

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use galarie_backend::{
    cache::CacheStore,
    config::{AppConfig, LogConfig, OtelConfig},
    indexer::Indexer,
    routes::{self, AppState},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tempfile::tempdir;
use tokio::sync::RwLock;
use tower::ServiceExt;

#[tokio::test]
async fn cache_miss_rebuilds_and_search_responds_under_one_second() {
    let media_root = sample_media_root();
    let cache_dir = tempdir().expect("temp cache dir");
    let config = Arc::new(test_config(
        media_root.clone(),
        cache_dir.path().to_path_buf(),
    ));

    let cache_store = Arc::new(CacheStore::new(cache_dir.path()));
    let rebuild_start = Instant::now();
    let scan_root = media_root.clone();
    let snapshot = cache_store
        .load_or_rebuild(|| Indexer::scan_once(&scan_root))
        .expect("cache rebuild");
    let rebuild_elapsed = rebuild_start.elapsed();
    assert!(
        rebuild_elapsed <= Duration::from_secs(1),
        "expected cache rebuild within 1s for sample dataset, took {rebuild_elapsed:?}"
    );

    let snapshot_state = Arc::new(RwLock::new(snapshot));
    let state = AppState::new(config.clone(), cache_store.clone(), snapshot_state);
    let app = routes::router(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?tags=sunset")
        .body(Body::empty())
        .expect("request");

    let response = app.clone().oneshot(request).await.expect("router response");
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "search endpoint should respond successfully"
    );

    let body = response.into_body().collect().await.expect("body");
    let json: Value = serde_json::from_slice(&body.to_bytes()).expect("json payload");
    assert!(
        json["items"].is_array(),
        "search response should contain items array"
    );
}

fn sample_media_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sample-media")
}

fn test_config(media_root: PathBuf, cache_dir: PathBuf) -> AppConfig {
    AppConfig {
        media_root,
        cache_dir,
        listen_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
        environment: "test".into(),
        otel: OtelConfig {
            endpoint: None,
            service_name: "test-backend".into(),
            disable_traces: true,
            disable_logs: true,
        },
        log: LogConfig {
            level: "info".into(),
        },
        cors_allowed_origins: Vec::new(),
        frontend_dist_dir: None,
    }
}
