pub use std::sync::Arc;
mod channel;
mod http_filter;
mod runtime;

pub use http_filter::HttpFilter;
pub use runtime::Runtime;

use crate::{blueprint, HttpIO};

pub fn init_http(http: impl HttpIO, script: blueprint::Script) -> Arc<dyn HttpIO> {
    log::debug!("Initializing JavaScript HTTP filter: {}", script.source);
    let script_io = Runtime::new(script);
    Arc::new(HttpFilter::new(http, script_io))
}
