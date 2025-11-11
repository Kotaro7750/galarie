use anyhow::Result;
use opentelemetry::{KeyValue, global, trace::TracerProvider as _};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    self as sdk,
    logs::{SdkLogger, SdkLoggerProvider},
    resource::Resource,
};
use tracing::{info, warn};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;

pub struct TelemetryGuard {
    tracer_provider: Option<sdk::trace::SdkTracerProvider>,
    logger_provider: Option<SdkLoggerProvider>,
}

impl TelemetryGuard {
    pub fn init(config: &AppConfig) -> Result<Self> {
        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&config.log.level))
            .unwrap_or_else(|_| EnvFilter::new("info"));

        match build_otel_pipelines(config)? {
            Some(pipelines) => {
                tracing_subscriber::registry()
                    .with(pipelines.trace_layer)
                    .with(pipelines.log_layer)
                    .with(env_filter)
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_target(false)
                            .with_file(false)
                            .with_line_number(false)
                            .json(),
                    )
                    .try_init()?;
                info!("OpenTelemetry tracing/log export enabled (json stdout retained)");
                Ok(Self {
                    tracer_provider: Some(pipelines.tracer_provider),
                    logger_provider: Some(pipelines.logger_provider),
                })
            }
            None => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_target(false)
                            .with_file(false)
                            .with_line_number(false)
                            .json(),
                    )
                    .try_init()?;
                Ok(Self {
                    tracer_provider: None,
                    logger_provider: None,
                })
            }
        }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.tracer_provider.take() {
            if let Err(err) = provider.shutdown() {
                warn!(error = ?err, "failed to shutdown tracer provider cleanly");
            }
        }
        if let Some(provider) = self.logger_provider.take() {
            if let Err(err) = provider.shutdown() {
                warn!(error = ?err, "failed to shutdown logger provider cleanly");
            }
        }
    }
}

struct OtelPipelines {
    trace_layer: OpenTelemetryLayer<Registry, sdk::trace::Tracer>,
    tracer_provider: sdk::trace::SdkTracerProvider,
    log_layer: OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger>,
    logger_provider: SdkLoggerProvider,
}

fn build_otel_pipelines(config: &AppConfig) -> Result<Option<OtelPipelines>> {
    let endpoint = match &config.otel.endpoint {
        Some(endpoint) if !endpoint.trim().is_empty() => endpoint.clone(),
        _ => return Ok(None),
    };

    let resource = Resource::builder()
        .with_service_name(config.otel.service_name.clone())
        .with_attribute(KeyValue::new(
            "deployment.environment.name",
            config.environment.clone(),
        ))
        .build();

    let span_exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .build()?;

    let provider = sdk::trace::SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(span_exporter)
        .build();

    let tracer = provider.tracer(config.otel.service_name.clone());
    global::set_tracer_provider(provider.clone());

    let log_exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(log_exporter)
        .build();
    let log_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    Ok(Some(OtelPipelines {
        trace_layer: tracing_opentelemetry::layer().with_tracer(tracer),
        tracer_provider: provider,
        log_layer,
        logger_provider,
    }))
}
