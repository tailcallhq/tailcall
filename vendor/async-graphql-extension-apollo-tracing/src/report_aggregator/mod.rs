use std::{collections::HashMap, time::Duration};

use futures::{
    channel::mpsc::{self, Sender},
    StreamExt,
};
use protobuf::Message;

use crate::{
    proto::reports::{Report, ReportHeader, Trace, TracesAndStats},
    packages::uname,
    runtime::{abort, spawn, Instant, JoinHandle},
};

/// The [ReportAggregator] is the structure which control the background task spawned to aggregate
/// and send data through Apollo Studio by constructing [Report] ready to be send
pub struct ReportAggregator {
    #[allow(dead_code)]
    handle: JoinHandle<()>,
    sender: Sender<(String, Trace)>,
}

const REPORTING_URL: &str = "https://usage-reporting.api.apollographql.com/api/ingress/traces";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const TARGET_LOG: &str = "apollo-studio-extension";
const BUFFER_SLOTS: usize = 32;
const MAX_TRACES: usize = 64;

impl ReportAggregator {
    pub fn initialize(
        authorization_token: String,
        hostname: String,
        graph_id: String,
        variant: String,
        service_version: String,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<(String, Trace)>(BUFFER_SLOTS);

        let reported_header = ReportHeader {
            uname: uname::uname()
                .ok()
                .unwrap_or_else(|| "No uname provided".to_string()),
            hostname,
            graph_ref: format!("{graph_id}@{variant}"),
            service_version,
            agent_version: format!("async-studio-extension-{}", VERSION),
            runtime_version: "Rust".to_string(),
            executable_schema_id: graph_id,
            special_fields: Default::default(),
        };

        let handle = spawn(async move {
            let client = reqwest::Client::new();

            let mut hashmap: HashMap<String, TracesAndStats> = HashMap::with_capacity(MAX_TRACES);

            let mut count = 0;
            let mut now = Instant::now();

            while let Some((name, trace)) = rx.next().await {
                trace!(target: TARGET_LOG, message = "Trace registered", trace = ?trace, name = ?name);
                match hashmap.get_mut(&name) {
                    Some(previous) => {
                        previous.trace.push(trace);
                    }
                    None => {
                        let mut trace_and_stats = TracesAndStats::default();
                        trace_and_stats.trace.push(trace);
                        hashmap.insert(name, trace_and_stats);
                    }
                }

                count += 1;

                if count > MAX_TRACES || now.elapsed() > Duration::from_secs(5) {
                    now = Instant::now();
                    use tracing::{field, field::debug, span, Level};

                    let span_batch = span!(
                        Level::DEBUG,
                        "Sending traces by batch to Apollo Studio",
                        response = field::Empty,
                        batched = ?count,
                    );

                    span_batch.in_scope(|| {
                        trace!(target: TARGET_LOG, message = "Sending traces by batch");
                    });

                    let hashmap_to_send = hashmap;
                    hashmap = HashMap::with_capacity(MAX_TRACES);

                    let report: Report = Report {
                        traces_pre_aggregated: false,
                        traces_per_query: hashmap_to_send,
                        header: Some(reported_header.clone()).into(),
                        ..Default::default()
                    };

                    let msg = report.write_to_bytes().unwrap();

                    let mut client = client
                        .post(REPORTING_URL)
                        .header("content-type", "application/protobuf")
                        .header("accept", "application/json")
                        .header("X-Api-Key", &authorization_token);

                    if cfg!(feature = "compression") {
                        client = client.header("content-encoding", "gzip");
                    };

                    let msg = match crate::compression::compress(msg) {
                        Ok(result) => result,
                        Err(e) => {
                            error!(target: TARGET_LOG, message = "An issue happened while GZIP compression", err = ?e);
                            continue;
                        }
                    };

                    let result = client.body(msg).send().await;

                    match result {
                        Ok(data) => {
                            span_batch.record("response", &debug(&data));
                            let text = data.text().await;
                            info!(target: TARGET_LOG, data = ?text);
                        }
                        Err(err) => {
                            let status_code = err.status();
                            error!(target: TARGET_LOG, status = ?status_code, error = ?err);
                        }
                    }
                }
            }
        });

        Self { handle, sender: tx }
    }

    pub fn sender(&self) -> Sender<(String, Trace)> {
        self.sender.clone()
    }
}

impl Drop for ReportAggregator {
    fn drop(&mut self) {
        abort(&self.handle);
        // TODO: Wait for the proper aborted task
    }
}
