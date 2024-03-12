use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;
use opentelemetry_sdk::testing::trace::InMemorySpanExporter;
use opentelemetry_sdk::trace::{Tracer, TracerProvider};
use tailcall::tracing::{default_tracing, filter_target};
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

use super::in_memory::InMemoryTelemetry;

fn set_trace_provider(exporter: InMemorySpanExporter) -> OpenTelemetryLayer<Registry, Tracer> {
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_config(opentelemetry_sdk::trace::config())
        .build();
    let tracer = provider.tracer("tracing");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    global::set_tracer_provider(provider);

    telemetry
}

fn set_meter_provider(exporter: InMemoryMetricsExporter) -> PeriodicReader {
    let reader: PeriodicReader = PeriodicReader::builder(exporter, Tokio).build();

    let provider = MeterProviderBuilder::default()
        .with_reader(reader.clone())
        .build();
    global::set_meter_provider(provider);

    reader
}

fn set_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // ignore errors since there is only one possible error when the global
    // subscriber is already set. The init is called multiple times in the same
    // process many times inside tests, so we want to ignore if it is already
    // set
    let _ = tracing::subscriber::set_global_default(subscriber);
}

pub fn init_opentelemetry() -> InMemoryTelemetry {
    let trace_exporter = InMemorySpanExporter::default();
    let metrics_exporter = InMemoryMetricsExporter::default();
    let trace_layer = set_trace_provider(trace_exporter.clone());
    let metrics_reader = set_meter_provider(metrics_exporter.clone());

    let subscriber = tracing_subscriber::registry()
        .with(trace_layer)
        .with(default_tracing().with_filter(filter_target("execution_spec")));

    set_tracing_subscriber(subscriber);

    InMemoryTelemetry { trace_exporter, metrics_exporter, metrics_reader }
}
