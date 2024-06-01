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
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::{Tracer, TracerProvider};
use opentelemetry_sdk::{runtime, Resource};
use serde::Serialize;
use tonic::metadata::MetadataMap;
use tracing::level_filters::LevelFilter;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::dynamic_filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

use super::metrics::init_metrics;
use crate::cli::CLIError;
use crate::core::blueprint::telemetry::{OtlpExporter, Telemetry, TelemetryExporter};
use crate::core::runtime::TargetRuntime;
use crate::core::tracing::{default_tracing_tailcall, get_log_level, tailcall_filter_target};

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
    exporter: &TelemetryExporter,
) -> TraceResult<Option<OpenTelemetryLayer<Registry, Tracer>>> {
    let provider = match exporter {
        TelemetryExporter::Stdout(config) => TracerProvider::builder()
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
        TelemetryExporter::Otlp(config) => opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter(config))
            .with_trace_config(opentelemetry_sdk::trace::config().with_resource(RESOURCE.clone()))
            .install_batch(runtime::Tokio)?
            .provider()
            .ok_or(TraceError::Other(
                anyhow!("Failed to instantiate OTLP provider").into(),
            ))?,
        // Prometheus works only with metrics
        TelemetryExporter::Prometheus(_) => return Ok(None),
        TelemetryExporter::Apollo(_) => return Ok(None),
    };
    let tracer = provider.tracer("tracing");
    let telemetry = tracing_opentelemetry::layer()
        .with_location(false)
        .with_threads(false)
        .with_tracer(tracer);

    global::set_tracer_provider(provider);

    Ok(Some(telemetry))
}

fn set_logger_provider(
    exporter: &TelemetryExporter,
) -> LogResult<Option<OpenTelemetryTracingBridge<LoggerProvider, Logger>>> {
    let provider = match exporter {
        TelemetryExporter::Stdout(config) => LoggerProvider::builder()
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
        TelemetryExporter::Otlp(config) => opentelemetry_otlp::new_pipeline()
            .logging()
            .with_exporter(otlp_exporter(config))
            .with_log_config(opentelemetry_sdk::logs::config().with_resource(RESOURCE.clone()))
            .install_batch(runtime::Tokio)?
        ,
        // Prometheus works only with metrics
        TelemetryExporter::Prometheus(_) => return Ok(None),
        TelemetryExporter::Apollo(_) => return Ok(None),
    };

    let otel_tracing_appender = OpenTelemetryTracingBridge::new(&provider);

    Ok(Some(otel_tracing_appender))
}

fn set_meter_provider(exporter: &TelemetryExporter) -> MetricsResult<()> {
    let provider = match exporter {
        TelemetryExporter::Stdout(config) => {
            let mut builder = opentelemetry_stdout::MetricsExporterBuilder::default();

            if config.pretty {
                builder = builder.with_encoder(|writer, data| {
                    pretty_encoder(writer, data).map_err(|err| MetricsError::Other(err.to_string()))
                })
            }

            let exporter = builder.build();
            let reader = PeriodicReader::builder(exporter, Tokio).build();

            MeterProviderBuilder::default()
                .with_reader(reader)
                .with_resource(RESOURCE.clone())
                .build()
        }
        TelemetryExporter::Otlp(config) => opentelemetry_otlp::new_pipeline()
            .metrics(Tokio)
            .with_resource(RESOURCE.clone())
            .with_exporter(otlp_exporter(config))
            .build()?,
        TelemetryExporter::Prometheus(_) => {
            let exporter = opentelemetry_prometheus::exporter()
                .with_registry(prometheus::default_registry().clone())
                .build()?;

            MeterProviderBuilder::default()
                .with_resource(RESOURCE.clone())
                .with_reader(exporter)
                .build()
        }
        _ => return Ok(()),
    };

    global::set_meter_provider(provider);

    Ok(())
}

fn set_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // ignore errors since there is only one possible error when the global
    // subscriber is already set. The init is called multiple times in the same
    // process inside tests, so we want to ignore if it is already set
    let _ = tracing::subscriber::set_global_default(subscriber);
}

pub fn init_opentelemetry(config: Telemetry, runtime: &TargetRuntime) -> anyhow::Result<()> {
    if let Some(export) = &config.export {
        global::set_error_handler(|error| {
            if !matches!(
                error,
                // ignore errors related to _Signal_(Other(ChannelFull))
                // that happens on high number of signals generated
                // when mpsc::channel size exceeds
                // TODO: increase the default size of channel for providers if required
                global::Error::Trace(TraceError::Other(_))
                    | global::Error::Metric(MetricsError::Other(_))
                    | global::Error::Log(LogError::Other(_)),
            ) {
                tracing::subscriber::with_default(default_tracing_tailcall(), || {
                    let cli = crate::cli::CLIError::new("Open Telemetry Error")
                        .caused_by(vec![CLIError::new(error.to_string().as_str())])
                        .trace(vec!["schema".to_string(), "@telemetry".to_string()]);
                    tracing::error!("{}", cli.color(true));
                });
            }
        })?;

        let trace_layer = set_trace_provider(export)?;
        let log_layer = set_logger_provider(export)?;
        set_meter_provider(export)?;

        global::set_text_map_propagator(TraceContextPropagator::new());

        let subscriber = tracing_subscriber::registry()
            .with(trace_layer)
            .with(
                log_layer.with_filter(dynamic_filter_fn(|_metatada, context| {
                    // ignore logs that are generated inside tracing::Span since they will be logged
                    // anyway with tracer_provider and log here only the events without associated
                    // span
                    context.lookup_current().is_none()
                })),
            )
            .with(tailcall_filter_target())
            .with(LevelFilter::from_level(
                get_log_level().unwrap_or(tracing::Level::INFO),
            ));

        init_metrics(runtime)?;

        set_tracing_subscriber(subscriber);
    } else {
        set_tracing_subscriber(default_tracing_tailcall());
    }

    Ok(())
}
