use std::{sync::Arc, time::Instant};

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;

/// Shared application state cloned into each request handler.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub boot_instant: Instant,
}

impl AppState {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            boot_instant: Instant::now(),
        }
    }
}

/// Build the Axum router with shared layers and routes.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

/// JSON payload returned by `/healthz`.
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    media_root: String,
    cache_dir: String,
    uptime_seconds: f64,
}

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        media_root: state.config.media_root.display().to_string(),
        cache_dir: state.config.cache_dir.display().to_string(),
        uptime_seconds: state.boot_instant.elapsed().as_secs_f64(),
    })
}
