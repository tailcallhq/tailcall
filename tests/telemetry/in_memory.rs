use anyhow::Result;
use opentelemetry_sdk::export::trace::SpanData;
use opentelemetry_sdk::metrics::data::{Metric, ResourceMetrics, ScopeMetrics};
use opentelemetry_sdk::metrics::reader::MetricReader;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;
use opentelemetry_sdk::testing::trace::InMemorySpanExporter;
use serde::{Deserialize, Serialize};

pub struct InMemoryTelemetry {
    pub(super) trace_exporter: InMemorySpanExporter,
    pub(super) metrics_reader: PeriodicReader,
    pub(super) metrics_exporter: InMemoryMetricsExporter,
}

impl InMemoryTelemetry {
    pub fn reset(&self) {
        self.trace_exporter.reset();
        self.metrics_exporter.reset();
    }

    pub fn get_traces(&self) -> Result<Vec<TestSpan>> {
        let spans = self.trace_exporter.get_finished_spans()?;

        Ok(spans.into_iter().map(TestSpan::from).collect())
    }

    pub async fn get_metrics(&self) -> Result<Vec<TestMetrics>> {
        let metrics_reader = self.metrics_reader.clone();
        // call force_flush from blocking task to prevent deadlocking
        // see https://github.com/open-telemetry/opentelemetry-rust/issues/1395
        tokio::task::spawn_blocking(move || metrics_reader.force_flush()).await??;

        let metrics = self.metrics_exporter.get_finished_metrics()?;

        let mut metrics: Vec<_> = metrics
            .into_iter()
            .map(TestMetrics::from)
            .filter(|v| !v.0.is_empty())
            .collect();

        // dedup the same data from metrics vec since some of them just generated on
        // start or during execution and they do not relate to actual test
        // running
        metrics.dedup();

        Ok(metrics)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSpan {
    name: String,
}

impl From<SpanData> for TestSpan {
    fn from(value: SpanData) -> Self {
        Self { name: value.name.into_owned() }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestMetricsEntry {
    name: String,
}

impl From<Metric> for TestMetricsEntry {
    fn from(value: Metric) -> Self {
        Self { name: value.name.into_owned() }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestScopedMetrics {
    name: String,
    entries: Vec<TestMetricsEntry>,
}

impl PartialOrd for TestScopedMetrics {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TestScopedMetrics {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl From<ScopeMetrics> for TestScopedMetrics {
    fn from(value: ScopeMetrics) -> Self {
        Self {
            name: value.scope.name.into_owned(),
            entries: value
                .metrics
                .into_iter()
                .map(TestMetricsEntry::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestMetrics(Vec<TestScopedMetrics>);

impl From<ResourceMetrics> for TestMetrics {
    fn from(value: ResourceMetrics) -> Self {
        let mut v: Vec<_> = value
            .scope_metrics
            .into_iter()
            .map(TestScopedMetrics::from)
            .collect();

        v.sort();

        Self(v)
    }
}
