pub mod metrics;

use std::io::Write;

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use opentelemetry::logs::{LogError, LogResult};
use opentelemetry::metrics::{MetricsError, Result as MetricsResult};
use opentelemetry::trace::{TraceError, TraceResult, TracerProvider as _};
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{TonicExporterBuilder, WithExportConfig};
use opentelemetry_sdk::logs::{Logger, LoggerProvider};
use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::{Tracer, TracerProvider};
use opentelemetry_sdk::{runtime, Resource};
use serde::Serialize;
use tonic::metadata::MetadataMap;
use tracing::{level_filters::LevelFilter, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::dynamic_filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

use self::metrics::init_metrics;
use crate::blueprint::opentelemetry::{Opentelemetry, OpentelemetryExporter, OtlpExporter};
use crate::runtime::TargetRuntime;
use crate::tracing::{default_filter_target, default_tracing};

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::default().merge(&Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "tailcall",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            option_env!("APP_VERSION").unwrap_or("dev"),
        ),
    ]))
});

fn pretty_encoder<T: Serialize>(writer: &mut dyn Write, data: T) -> Result<()> {
    // convert to buffer first to use write_all and minimize
    // interleaving for std stream output
    let buf = serde_json::to_vec_pretty(&data)?;

    Ok(writer.write_all(&buf)?)
}

// TODO: add more options for otlp exporter if needed
fn otlp_exporter(config: &OtlpExporter) -> TonicExporterBuilder {
    opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(config.url.as_str())
        .with_metadata(MetadataMap::from_headers(config.headers.clone()))
}

fn set_trace_provider(
    exporter: &OpentelemetryExporter,
) -> TraceResult<Option<OpenTelemetryLayer<Registry, Tracer>>> {
    let provider = match exporter {
        OpentelemetryExporter::Stdout(config) => TracerProvider::builder()
            .with_batch_exporter(
                {
                    let mut builder = opentelemetry_stdout::SpanExporterBuilder::default();

                    if config.pretty {
                        builder = builder.with_encoder(|writer, data| {
                            pretty_encoder(writer, data)
                                .map_err(|err| TraceError::Other(err.into()))
                        })
                    }

                    builder.build()
                },
                runtime::Tokio,
            )
            .with_config(opentelemetry_sdk::trace::config().with_resource(RESOURCE.clone()))
            .build(),
        OpentelemetryExporter::Otlp(config) => opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter(config))
            .with_trace_config(opentelemetry_sdk::trace::config().with_resource(RESOURCE.clone()))
            .install_batch(runtime::Tokio)?
            .provider()
            .ok_or(TraceError::Other(
                anyhow!("Failed to instantiate OTLP provider").into(),
            ))?,
    };
    let tracer = provider.tracer("tracing");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    global::set_tracer_provider(provider);

    Ok(Some(telemetry))
}

fn set_logger_provider(
    exporter: &OpentelemetryExporter,
) -> LogResult<Option<OpenTelemetryTracingBridge<LoggerProvider, Logger>>> {
    let provider = match exporter {
        OpentelemetryExporter::Stdout(config) => LoggerProvider::builder()
            .with_batch_exporter(
                {
                    let mut builder = opentelemetry_stdout::LogExporterBuilder::default();

                    if config.pretty {
                        builder = builder.with_encoder(|writer, data| {
                            pretty_encoder(writer, data).map_err(|err| LogError::Other(err.into()))
                        })
                    }

                    builder.build()
                },
                runtime::Tokio,
            )
            .with_config(opentelemetry_sdk::logs::config().with_resource(RESOURCE.clone()))
            .build(),
        OpentelemetryExporter::Otlp(config) => opentelemetry_otlp::new_pipeline()
            .logging()
            .with_exporter(otlp_exporter(config))
            .with_log_config(opentelemetry_sdk::logs::config().with_resource(RESOURCE.clone()))
            .install_batch(runtime::Tokio)?
            .provider()
            .ok_or(LogError::Other(
                anyhow!("Failed to instantiate OTLP provider").into(),
            ))?,
    };

    let otel_tracing_appender = OpenTelemetryTracingBridge::new(&provider);

    global::set_logger_provider(provider);

    Ok(Some(otel_tracing_appender))
}

fn set_meter_provider(exporter: &OpentelemetryExporter) -> MetricsResult<()> {
    let provider = match exporter {
        OpentelemetryExporter::Stdout(config) => {
            let mut builder = opentelemetry_stdout::MetricsExporterBuilder::default();

            if config.pretty {
                builder = builder.with_encoder(|writer, data| {
                    pretty_encoder(writer, data).map_err(|err| MetricsError::Other(err.to_string()))
                })
            }

            let exporter = builder.build();
            let reader = PeriodicReader::builder(exporter, Tokio).build();

            MeterProvider::builder()
                .with_reader(reader)
                .with_resource(RESOURCE.clone())
                .build()
        }
        OpentelemetryExporter::Otlp(config) => opentelemetry_otlp::new_pipeline()
            .metrics(Tokio)
            .with_resource(RESOURCE.clone())
            .with_exporter(otlp_exporter(config))
            .build()?,
    };

    global::set_meter_provider(provider);

    Ok(())
}

fn set_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // ignore errors since there is only one possible error when the global subscriber
    // is already set. The init is called multiple times in the same process inside
    // tests, so we want to ignore if it is already set
    let _ = tracing::subscriber::set_global_default(subscriber);
}

pub fn init_opentelemetry(config: Opentelemetry, runtime: &TargetRuntime) -> anyhow::Result<()> {
    if let Some(config) = &config.0 {
        global::set_error_handler(|_| {
            // TODO: do something with the error
            // by default it's printed to stderr
        })?;

        let trace_layer = set_trace_provider(&config.export)?;
        let log_layer = set_logger_provider(&config.export)?;
        set_meter_provider(&config.export)?;

        let subscriber = tracing_subscriber::registry()
            .with(trace_layer.with_filter(LevelFilter::INFO))
            .with(
                log_layer.with_filter(dynamic_filter_fn(|_metatada, context| {
                    // ignore logs that are generated inside tracing::Span since they will be logged
                    // anyway with tracer_provider and log here only the events without associated span
                    context.lookup_current().is_none()
                })),
            )
            .with(default_filter_target());

        init_metrics(runtime)?;

        set_tracing_subscriber(subscriber)
    } else {
        set_tracing_subscriber(default_tracing());
    }

    Ok(())
}
