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

    /// Disable OTLP trace export even if an endpoint is set
    #[arg(long, env = "GALARIE_OTEL_DISABLE_TRACES", default_value_t = false)]
    otel_disable_traces: bool,

    /// Disable OTLP log export even if an endpoint is set
    #[arg(long, env = "GALARIE_OTEL_DISABLE_LOGS", default_value_t = false)]
    otel_disable_logs: bool,

    /// Deployment environment tag for telemetry (e.g., development, staging, prod)
    #[arg(long, env = "GALARIE_ENV", default_value = "development")]
    environment: String,

    /// Default log filter when RUST_LOG is not provided
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    log_level: String,

    /// Comma-separated list of allowed CORS origins
    #[arg(long, env = "GALARIE_CORS_ALLOWED_ORIGINS", value_delimiter = ',')]
    cors_allowed_origins: Vec<String>,

    /// Directory containing the built frontend assets
    #[arg(long, env = "GALARIE_FRONTEND_DIST_DIR")]
    frontend_dist_dir: Option<PathBuf>,
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
    pub cors_allowed_origins: Vec<String>,
    pub frontend_dist_dir: Option<PathBuf>,
}

/// OpenTelemetry exporter configuration.
#[derive(Debug, Clone)]
pub struct OtelConfig {
    pub endpoint: Option<String>,
    pub service_name: String,
    pub disable_traces: bool,
    pub disable_logs: bool,
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
        ensure_binary_exists("ffmpeg")
            .context("required dependency 'ffmpeg' was not found in PATH")?;
        ensure_binary_exists("gifsicle")
            .context("required dependency 'gifsicle' was not found in PATH")?;

        let frontend_dist_dir = value.frontend_dist_dir.clone();
        if let Some(dir) = &frontend_dist_dir {
            ensure_directory_exists(dir)
                .with_context(|| format!("frontend dist directory '{}' missing", dir.display()))?;
        }

        Ok(Self {
            media_root: value.media_root,
            cache_dir: value.cache_dir,
            listen_addr: value.listen_addr,
            environment: value.environment,
            otel: OtelConfig {
                endpoint: value.otel_endpoint,
                service_name: value.otel_service_name,
                disable_traces: value.otel_disable_traces,
                disable_logs: value.otel_disable_logs,
            },
            log: LogConfig {
                level: value.log_level,
            },
            cors_allowed_origins: value
                .cors_allowed_origins
                .into_iter()
                .filter(|origin| !origin.is_empty())
                .collect(),
            frontend_dist_dir,
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

fn ensure_binary_exists(binary: &str) -> Result<()> {
    which::which(binary)
        .map(|_| ())
        .with_context(|| format!("binary '{}' is required but was not found in PATH", binary))
}
