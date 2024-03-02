use std::sync::Arc;

use crate::{blueprint, HttpIO};

mod http_filter;
mod runtime;

pub use http_filter::HttpFilter;
pub use runtime::Runtime;

pub fn init_http(http: impl HttpIO, script: blueprint::Script) -> Arc<dyn HttpIO + Sync + Send> {
    log::debug!("Initializing JavaScript HTTP filter: {}", script.source);
    let script_io = Runtime::new(script);
    Arc::new(HttpFilter::new(http, script_io))
}
