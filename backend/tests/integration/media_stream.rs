use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    vec::Vec,
};

use axum::{
    Router,
    body::Body,
    http::{
        Method, Request, StatusCode,
        header::{ACCEPT_RANGES, CONTENT_TYPE, ETAG},
    },
};
use galarie_backend::{
    cache::CacheStore,
    config::{AppConfig, LogConfig, OtelConfig},
    indexer::{Indexer, MediaFile, MediaType},
    routes::{self, AppState},
};
use http_body_util::BodyExt;
use tempfile::tempdir;
use tokio::{fs, sync::RwLock};
use tower::ServiceExt;

#[tokio::test]
#[ignore = "stream endpoint not implemented yet"]
async fn stream_returns_original_bytes_with_headers() {
    let ctx = StreamTestContext::new(MediaType::Image).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/media/{}/stream", ctx.media.id))
        .body(Body::empty())
        .expect("request");

    let response = ctx
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(ACCEPT_RANGES).unwrap(),
        "bytes",
        "range header required for seeking"
    );
    let content_type = response.headers().get(CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "image/png");
    let etag = response.headers().get(ETAG).expect("etag header present");
    assert!(
        !etag.is_empty(),
        "stream responses must emit cache validators"
    );

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body bytes")
        .to_bytes();

    let expected_path = ctx.media_root.join(Path::new(&ctx.media.relative_path));
    let expected = fs::read(expected_path)
        .await
        .expect("read sample media file");
    assert_eq!(body, expected);
}

#[tokio::test]
#[ignore = "stream endpoint not implemented yet"]
async fn missing_media_returns_not_found() {
    let ctx = StreamTestContext::new(MediaType::Image).await;
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media/deadbeef/stream")
        .body(Body::empty())
        .expect("request");

    let response = ctx
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = response.into_body().collect().await.expect("body");
    let json: serde_json::Value = serde_json::from_slice(&body.to_bytes()).expect("json payload");
    assert_eq!(json["error"]["code"], "RESOURCE_NOT_FOUND");
    assert!(
        json["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("media"),
        "not-found errors should mention the missing media"
    );
}

struct StreamTestContext {
    media_root: PathBuf,
    media: MediaFile,
    router: Router,
}

impl StreamTestContext {
    async fn new(target_type: MediaType) -> Self {
        let media_root = sample_media_root();
        let cache_dir = tempdir().expect("temp cache dir");
        let cache_store = Arc::new(CacheStore::new(cache_dir.path()));
        let config = Arc::new(test_config(
            media_root.clone(),
            cache_dir.path().to_path_buf(),
        ));

        let scan_root = media_root.clone();
        let snapshot = cache_store
            .load_or_rebuild(|| Indexer::scan_once(&scan_root))
            .expect("cache rebuild");
        let media = snapshot
            .media
            .iter()
            .find(|item| item.media_type == target_type)
            .cloned()
            .expect("sample media for requested type");
        let snapshot_state = Arc::new(RwLock::new(snapshot));
        let state = AppState::new(config, cache_store, snapshot_state);
        let router = routes::router(state);

        Self {
            media_root,
            media,
            router,
        }
    }
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
        },
        log: LogConfig {
            level: "info".into(),
        },
        cors_allowed_origins: Vec::new(),
    }
}
