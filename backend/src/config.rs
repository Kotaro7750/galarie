use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use clap::Parser;

/// CLI / env configuration parsed at process startup.
#[derive(Debug, Clone, Parser)]
#[command(
    name = "galarie-backend",
    about = "Filesystem-backed media search API",
    version,
    disable_help_subcommand = true
)]
struct CliConfig {
    /// Root directory containing tagged media files
    #[arg(long, env = "GALARIE_MEDIA_ROOT")]
    media_root: PathBuf,

    /// Directory for cache/temporary data
    #[arg(long, env = "GALARIE_CACHE_DIR", default_value = "./.cache")]
    cache_dir: PathBuf,

    /// Address to bind the HTTP server to (e.g., 0.0.0.0:8080)
    #[arg(long, env = "GALARIE_BIND_ADDR", default_value = "0.0.0.0:8080")]
    listen_addr: SocketAddr,

    /// Optional OTLP endpoint (grpc or http/proto) for OpenTelemetry export
    #[arg(long, env = "OTEL_EXPORTER_OTLP_ENDPOINT")]
    otel_endpoint: Option<String>,

    /// Logical service name for telemetry (resource attribute)
    #[arg(long, env = "OTEL_SERVICE_NAME", default_value = "galarie-backend")]
    otel_service_name: String,

    /// Deployment environment tag for telemetry (e.g., development, staging, prod)
    #[arg(long, env = "GALARIE_ENV", default_value = "development")]
    environment: String,

    /// Default log filter when RUST_LOG is not provided
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    log_level: String,
}

/// Fully validated configuration shared across the application.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub media_root: PathBuf,
    pub cache_dir: PathBuf,
    pub listen_addr: SocketAddr,
    pub otel: OtelConfig,
    pub log: LogConfig,
    pub environment: String,
}

/// OpenTelemetry exporter configuration.
#[derive(Debug, Clone)]
pub struct OtelConfig {
    pub endpoint: Option<String>,
    pub service_name: String,
}

/// Structured logging configuration.
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: String,
}

impl AppConfig {
    /// Parse CLI/env arguments and return a validated configuration.
    pub fn load() -> Result<Self> {
        let cli = CliConfig::parse();
        Self::try_from(cli)
    }
}

impl TryFrom<CliConfig> for AppConfig {
    type Error = anyhow::Error;

    fn try_from(value: CliConfig) -> Result<Self> {
        ensure_directory_exists(&value.media_root)
            .with_context(|| format!("media root '{}' missing", value.media_root.display()))?;
        fs::create_dir_all(&value.cache_dir).with_context(|| {
            format!("failed to create cache dir '{}'", value.cache_dir.display())
        })?;

        Ok(Self {
            media_root: value.media_root,
            cache_dir: value.cache_dir,
            listen_addr: value.listen_addr,
            environment: value.environment,
            otel: OtelConfig {
                endpoint: value.otel_endpoint,
                service_name: value.otel_service_name,
            },
            log: LogConfig {
                level: value.log_level,
            },
        })
    }
}

fn ensure_directory_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    Err(anyhow!(
        "path '{}' does not exist or is not accessible",
        path.display()
    ))
}
