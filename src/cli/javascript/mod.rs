mod http_filter;
mod runtime;
mod serde_v8;
mod shim;
mod worker;
pub use std::sync::Arc;

pub use http_filter::HttpFilter;
pub use runtime::Runtime;

use crate::{blueprint, HttpIO};

pub fn init_http(http: impl HttpIO, script: blueprint::Script) -> Arc<dyn HttpIO> {
    let script_io = Runtime::new(script);
    Arc::new(HttpFilter::new(http, script_io))
}
