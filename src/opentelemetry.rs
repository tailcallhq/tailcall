use opentelemetry::logs::LogError;
use opentelemetry::logs::LogResult;
use opentelemetry::metrics::Result as MetricsResult;
use opentelemetry::trace::TraceError;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, trace::TraceResult};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::{Logger, LoggerProvider};
use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::dynamic_filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

use crate::config::opentelemetry::OpenTelemetry;
use crate::config::opentelemetry::OpenTelemetryExporter;
use crate::tracing::default_filter_target;
use crate::tracing::default_tracing;

fn set_trace_provider(config: &OpenTelemetry) -> TraceResult<Option<OpenTelemetryLayer<Registry, Tracer>>> {
  let provider = match &config.export {
    OpenTelemetryExporter::None => {
      return Ok(None);
    }
    OpenTelemetryExporter::Stdout => TracerProvider::builder()
      // TODO: use batched exporter
      .with_simple_exporter(
        opentelemetry_stdout::SpanExporterBuilder::default()
          // TODO: use pretty config
          .with_encoder(|writer, data| {
            let buf = serde_json::to_vec_pretty(&data).map_err(|err| TraceError::Other(err.into()))?;

            writer.write_all(&buf).map_err(|err| TraceError::Other(err.into()))
          })
          // .with_encoder(|writer, data| Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
          .build(),
      )
      .build(),
    OpenTelemetryExporter::OTLP => {
      let exporter = opentelemetry_otlp::new_exporter().tonic();
      opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .install_simple()?
        .provider()
        .unwrap()
    }
  };
  let tracer = provider.tracer("tracing");
  let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

  global::set_tracer_provider(provider);

  Ok(Some(telemetry))
}

fn set_logger_provider(
  config: &OpenTelemetry,
) -> LogResult<Option<OpenTelemetryTracingBridge<LoggerProvider, Logger>>> {
  let provider = match &config.export {
    OpenTelemetryExporter::None => {
      return Ok(None);
    }
    OpenTelemetryExporter::Stdout => {
      LoggerProvider::builder()
        // TODO: use batched exporter
        .with_simple_exporter(
          opentelemetry_stdout::LogExporterBuilder::default()
            // TODO: use pretty config
            .with_encoder(|writer, data| {
              let buf = serde_json::to_vec_pretty(&data).map_err(|err| LogError::Other(err.into()))?;

              writer.write_all(&buf).map_err(|err| LogError::Other(err.into()))
            })
            // .with_encoder(|writer, data| Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
            .build(),
        )
        .build()
    }
    OpenTelemetryExporter::OTLP => {
      let exporter = opentelemetry_otlp::new_exporter().tonic();
      opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(exporter)
        .install_simple()?
        .provider()
        .unwrap()
    }
  };

  let otel_tracing_appender = OpenTelemetryTracingBridge::new(&provider);

  global::set_logger_provider(provider);

  Ok(Some(otel_tracing_appender))
}

fn set_meter_provider(config: &OpenTelemetry) -> MetricsResult<()> {
  let provider = match &config.export {
    OpenTelemetryExporter::None => {
      return Ok(());
    }
    OpenTelemetryExporter::Stdout => {
      let exporter = opentelemetry_stdout::MetricsExporterBuilder::default()
        // TODO: use pretty config
        // TODO: create as separate function
        // only to prevent output interleaving
        .with_encoder(|writer, data| Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
        .build();
      let reader = PeriodicReader::builder(exporter, Tokio).build();

      MeterProvider::builder().with_reader(reader).build()
    }
    OpenTelemetryExporter::OTLP => {
      let exporter = opentelemetry_otlp::new_exporter().tonic();
      opentelemetry_otlp::new_pipeline()
        .metrics(Tokio)
        .with_exporter(exporter)
        .build()?
    }
  };

  global::set_meter_provider(provider);

  Ok(())
}

// TODO: set global attributes
pub fn init_opentelemetry(config: OpenTelemetry) -> anyhow::Result<()> {
  let trace_layer = set_trace_provider(&config)?;
  let log_layer = set_logger_provider(&config)?;
  set_meter_provider(&config)?;

  if let OpenTelemetryExporter::None = config.export {
    default_tracing().try_init()?;
  } else {
    tracing_subscriber::registry()
      .with(trace_layer)
      .with(log_layer.with_filter(dynamic_filter_fn(|_metatada, context| {
        context.lookup_current().is_none()
      })))
      .with(default_filter_target())
      .try_init()?;
  }

  Ok(())
}
