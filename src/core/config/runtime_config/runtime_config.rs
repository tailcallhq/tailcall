use derive_setters::Setters;

use super::{ServerRuntime, TelemetryRuntime, UpstreamRuntime};

#[derive(Clone, Debug, Default, Setters)]
pub struct RuntimeConfig {
    pub server: ServerRuntime,
    pub upstream: UpstreamRuntime,
    pub telemetry: TelemetryRuntime,
}
