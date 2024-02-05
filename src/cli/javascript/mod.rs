mod http_filter_mv8;
mod runtime_mv8;
mod serde_mv8;
mod shim;
pub use std::sync::Arc;
mod channel_mv8;
mod deno_channel;
mod deno_http_filter;
mod deno_runtime;

pub use deno_http_filter::HttpFilter;
pub use deno_runtime::Runtime;

use crate::{blueprint, HttpIO};

pub fn init_http(http: impl HttpIO, script: blueprint::Script) -> Arc<dyn HttpIO> {
    let script_io = Runtime::new(script);
    Arc::new(HttpFilter::new(http, script_io))
}
