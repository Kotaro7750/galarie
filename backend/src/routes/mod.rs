use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Error;
use axum::{
    Json, Router,
    extract::{MatchedPath, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use serde::Serialize;
use tokio::{sync::RwLock, task};
use tower_http::trace::{MakeSpan, OnRequest, OnResponse, TraceLayer};
use tracing::{Instrument, Span, field, instrument};

use crate::{
    api::{self, ApiResponse, ApiResult},
    cache::{CacheSnapshot, CacheStore},
    config::{AppConfig, LogConfig, OtelConfig},
    indexer::Indexer,
};

/// Shared application state cloned into each request handler.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub cache_store: Arc<CacheStore>,
    pub snapshot: Arc<RwLock<CacheSnapshot>>,
    pub boot_instant: Instant,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        cache_store: Arc<CacheStore>,
        snapshot: Arc<RwLock<CacheSnapshot>>,
    ) -> Self {
        Self {
            config,
            cache_store,
            snapshot,
            boot_instant: Instant::now(),
        }
    }
}

/// Build the Axum router with shared layers and routes.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/v1/index/rebuild", post(trigger_rebuild))
        .with_state(state)
        .fallback(api::fallback_handler)
        .layer(middleware::from_fn(api::ensure_error_envelope))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(HttpMakeSpan)
                .on_request(LogOnRequest)
                .on_response(LogOnResponse),
        )
}

/// JSON payload returned by `/healthz`.
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    media_root: String,
    cache_dir: String,
    uptime_seconds: f64,
    cache_items: usize,
    cache_generated_at: String,
}

#[instrument(skip(state))]
async fn healthz(State(state): State<AppState>) -> ApiResult<HealthResponse> {
    let snapshot = state.snapshot.read().await;
    Ok(Json(HealthResponse {
        status: "ok",
        media_root: state.config.media_root.display().to_string(),
        cache_dir: state.config.cache_dir.display().to_string(),
        uptime_seconds: state.boot_instant.elapsed().as_secs_f64(),
        cache_items: snapshot.media.len(),
        cache_generated_at: snapshot.generated_at.to_rfc3339(),
    }))
}

#[instrument(skip(state))]
async fn trigger_rebuild(State(state): State<AppState>) -> ApiResponse<serde_json::Value> {
    let cache_store = state.cache_store.clone();
    let snapshot_state = state.snapshot.clone();
    let media_root = state.config.media_root.clone();

    task::spawn(async move {
        let span = tracing::info_span!("api_triggerred_index", media_root = %media_root.display());

        let root_for_scan = media_root.clone();
        if let Err(err) = async move {
            let parent = tracing::Span::current();
            let files = tokio::task::spawn_blocking(move || {
                parent.in_scope(|| Indexer::scan_once(&root_for_scan))
            })
            .await??;
            let snapshot = cache_store.persist(files)?;
            *snapshot_state.write().await = snapshot;
            Result::<(), Error>::Ok(())
        }
        .instrument(span)
        .await
        {
            tracing::error!(error = %err, "manual index rebuild failed");
        } else {
            tracing::info!("manual index rebuild completed");
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({"status": "queued"})),
    ))
}

#[derive(Clone)]
struct HttpMakeSpan;

impl<B> MakeSpan<B> for HttpMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> Span {
        let method = request.method().clone();
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|path| path.as_str())
            .unwrap_or_else(|| request.uri().path());

        let span = tracing::info_span!(
            "http_request",
            http.request.method = %method,
            http.route = %matched_path,
            url.path = request.uri().path(),
            url.query = field::Empty,
            http.response.status_code = field::Empty,
            http.latency_ms = field::Empty
        );

        if let Some(query) = request.uri().query() {
            span.record("url.query", &field::display(query));
        }

        span
    }
}

#[derive(Clone)]
struct LogOnRequest;

impl<B> OnRequest<B> for LogOnRequest {
    fn on_request(&mut self, request: &axum::http::Request<B>, span: &Span) {
        tracing::info!(
            parent: span,
            "HTTP request received: {} {}",
            request.method(),
            request.uri().path()
        );
    }
}

#[derive(Clone)]
struct LogOnResponse;

impl<B> OnResponse<B> for LogOnResponse {
    fn on_response(self, response: &axum::http::Response<B>, latency: Duration, span: &Span) {
        let status_code = response.status().as_u16();

        span.record("http.response.status_code", &field::display(status_code));
        span.record("http.latency_ms", &field::display(latency.as_millis()));

        tracing::info!(
            parent: span,
            "HTTP request completed with status {} in {} ms",
            status_code,
            latency.as_millis()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request},
    };
    use http_body_util::BodyExt;
    use serde_json::Value;
    use std::{fs, os::unix::fs::PermissionsExt, time::Duration};
    use tempfile::tempdir;
    use tokio::time::timeout;
    use tower::ServiceExt;

    fn sample_media_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sample-media")
    }

    fn test_config(media_root: PathBuf, cache_dir: PathBuf) -> AppConfig {
        AppConfig {
            media_root,
            cache_dir,
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            environment: "test".into(),
            otel: OtelConfig {
                endpoint: None,
                service_name: "test-service".into(),
            },
            log: LogConfig {
                level: "info".into(),
            },
        }
    }

    async fn post_rebuild(app: &mut Router) -> StatusCode {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/index/rebuild")
            .body(Body::empty())
            .unwrap();

        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn rebuild_endpoint_updates_cache_snapshot() {
        let media_root = sample_media_root();
        let cache_dir = tempdir().unwrap();
        let config = Arc::new(test_config(media_root, cache_dir.path().to_path_buf()));
        let cache_store = Arc::new(CacheStore::new(cache_dir.path()));
        let initial_snapshot = CacheSnapshot::new(Vec::new());
        let snapshot_state = Arc::new(RwLock::new(initial_snapshot));

        let state = AppState::new(config, cache_store.clone(), snapshot_state.clone());
        let mut app = router(state);

        let status = post_rebuild(&mut app).await;
        assert_eq!(status, StatusCode::ACCEPTED);

        timeout(Duration::from_secs(2), async {
            loop {
                if snapshot_state.read().await.media.len() >= 3 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("rebuild did not complete in time");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn rebuild_endpoint_handles_persist_failure() {
        let media_root = sample_media_root();
        let cache_dir = tempdir().unwrap();
        let config = Arc::new(test_config(media_root, cache_dir.path().to_path_buf()));
        let cache_store = Arc::new(CacheStore::new(cache_dir.path()));
        let initial_snapshot = CacheSnapshot::new(Vec::new());
        let snapshot_state = Arc::new(RwLock::new(initial_snapshot));

        fs::set_permissions(cache_dir.path(), fs::Permissions::from_mode(0o555)).unwrap();

        let state = AppState::new(config, cache_store, snapshot_state.clone());
        let mut app = router(state);

        let status = post_rebuild(&mut app).await;
        assert_eq!(status, StatusCode::ACCEPTED);

        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(snapshot_state.read().await.media.len(), 0);

        fs::set_permissions(cache_dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[tokio::test]
    async fn fallback_returns_standard_error() {
        let media_root = sample_media_root();
        let cache_dir = tempdir().unwrap();
        let config = Arc::new(test_config(media_root, cache_dir.path().to_path_buf()));
        let cache_store = Arc::new(CacheStore::new(cache_dir.path()));
        let snapshot_state = Arc::new(RwLock::new(CacheSnapshot::new(Vec::new())));

        let state = AppState::new(config, cache_store, snapshot_state);
        let app = router(state);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "RESOURCE_NOT_FOUND");
    }

    #[tokio::test]
    async fn method_not_allowed_returns_standard_error() {
        let media_root = sample_media_root();
        let cache_dir = tempdir().unwrap();
        let config = Arc::new(test_config(media_root, cache_dir.path().to_path_buf()));
        let cache_store = Arc::new(CacheStore::new(cache_dir.path()));
        let snapshot_state = Arc::new(RwLock::new(CacheSnapshot::new(Vec::new())));

        let state = AppState::new(config, cache_store, snapshot_state);
        let app = router(state);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/index/rebuild")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "METHOD_NOT_ALLOWED");
    }
}
