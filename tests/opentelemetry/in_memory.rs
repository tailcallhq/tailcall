use anyhow::Result;
use opentelemetry_sdk::export::trace::SpanData;
use opentelemetry_sdk::metrics::data::{Metric, ResourceMetrics, ScopeMetrics};
use opentelemetry_sdk::metrics::reader::MetricReader;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;
use opentelemetry_sdk::testing::trace::InMemorySpanExporter;
use serde::{Deserialize, Serialize};

pub struct InMemoryOpentelemetry {
    pub trace_exporter: InMemorySpanExporter,
    pub metrics_reader: PeriodicReader,
    pub metrics_exporter: InMemoryMetricsExporter,
}

impl InMemoryOpentelemetry {
    pub fn reset(&self) {
        self.trace_exporter.reset();
        self.metrics_exporter.reset();
    }

    pub fn get_traces(&self) -> Result<Vec<TestSpan>> {
        let spans = self.trace_exporter.get_finished_spans()?;

        Ok(spans.into_iter().map(TestSpan::from).collect())
    }

    pub fn get_metrics(&self) -> Result<Vec<TestMetrics>> {
        self.metrics_reader.force_flush()?;
        let metrics = self.metrics_exporter.get_finished_metrics()?;

        Ok(metrics.into_iter().map(TestMetrics::from).collect())
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TestMetricsEntry {
    name: String,
}

impl From<Metric> for TestMetricsEntry {
    fn from(value: Metric) -> Self {
        Self { name: value.name.into_owned() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestScopedMetrics {
    name: String,
    entries: Vec<TestMetricsEntry>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TestMetrics(Vec<TestScopedMetrics>);

impl From<ResourceMetrics> for TestMetrics {
    fn from(value: ResourceMetrics) -> Self {
        Self(
            value
                .scope_metrics
                .into_iter()
                .map(TestScopedMetrics::from)
                .collect(),
        )
    }
}
