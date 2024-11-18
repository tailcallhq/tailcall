use super::{ServerRuntime, TelemetryRuntime, UpstreamRuntime};

pub struct RuntimeConfig {
    pub server: ServerRuntime,
    pub upstream: UpstreamRuntime,
    pub telemetry: TelemetryRuntime,
}
