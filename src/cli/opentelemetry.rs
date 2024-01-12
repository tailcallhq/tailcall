use std::io::Write;

use anyhow::Result;
use opentelemetry::logs::LogError;
use opentelemetry::logs::LogResult;
use opentelemetry::metrics::MetricsError;
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
use serde::Serialize;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::dynamic_filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

use crate::config::opentelemetry::Opentelemetry;
use crate::config::opentelemetry::OpentelemetryExporter;
use crate::tracing::default_filter_target;
use crate::tracing::default_tracing;

fn pretty_encoder<T: Serialize>(writer: &mut dyn Write, data: T) -> Result<()> {
  // convert to buffer first to use write_all and minimize
  // interleaving std stream output
  let buf = serde_json::to_vec_pretty(&data)?;

  Ok(writer.write_all(&buf)?)
}

fn set_trace_provider(exporter: &OpentelemetryExporter) -> TraceResult<Option<OpenTelemetryLayer<Registry, Tracer>>> {
  let provider = match exporter {
    OpentelemetryExporter::Stdout { pretty } => TracerProvider::builder()
      // TODO: use batched exporter
      .with_simple_exporter({
        let mut builder = opentelemetry_stdout::SpanExporterBuilder::default();

        if *pretty {
          builder = builder
            .with_encoder(|writer, data| pretty_encoder(writer, data).map_err(|err| TraceError::Other(err.into())))
        }

        builder.build()
      })
      .build(),
    OpentelemetryExporter::Otlp => {
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
  exporter: &OpentelemetryExporter,
) -> LogResult<Option<OpenTelemetryTracingBridge<LoggerProvider, Logger>>> {
  let provider = match exporter {
    OpentelemetryExporter::Stdout { pretty } => {
      LoggerProvider::builder()
        // TODO: use batched exporter
        .with_simple_exporter({
          let mut builder = opentelemetry_stdout::LogExporterBuilder::default();

          if *pretty {
            builder = builder
              .with_encoder(|writer, data| pretty_encoder(writer, data).map_err(|err| LogError::Other(err.into())))
          }

          builder.build()
        })
        .build()
    }
    OpentelemetryExporter::Otlp => {
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

fn set_meter_provider(exporter: &OpentelemetryExporter) -> MetricsResult<()> {
  let provider = match exporter {
    OpentelemetryExporter::Stdout { pretty } => {
      let mut builder = opentelemetry_stdout::MetricsExporterBuilder::default();

      if *pretty {
        builder = builder
          .with_encoder(|writer, data| pretty_encoder(writer, data).map_err(|err| MetricsError::Other(err.to_string())))
      }

      let exporter = builder.build();
      let reader = PeriodicReader::builder(exporter, Tokio).build();

      MeterProvider::builder().with_reader(reader).build()
    }
    OpentelemetryExporter::Otlp => {
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

fn set_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
  // ignore errors since there is only one possible error when the global subscriber
  // is already set. The init is called multiple times in the same process many times inside
  // tests, so we want to ignore if it is already set
  let _ = tracing::subscriber::set_global_default(subscriber);
}

// TODO: set global attributes
pub fn init_opentelemetry(config: Opentelemetry) -> anyhow::Result<()> {
  if let Some(exporter) = &config.export {
    let trace_layer = set_trace_provider(exporter)?;
    let log_layer = set_logger_provider(exporter)?;
    set_meter_provider(exporter)?;
    let subscriber = tracing_subscriber::registry()
      .with(trace_layer)
      .with(log_layer.with_filter(dynamic_filter_fn(|_metatada, context| {
        // ignore logs that are generated inside tracing::Span since they will be logged
        // anyway with tracer_provider and log here only the events without associated span
        context.lookup_current().is_none()
      })))
      .with(default_filter_target());

    set_tracing_subscriber(subscriber)
  } else {
    set_tracing_subscriber(default_tracing());
  }

  Ok(())
}
