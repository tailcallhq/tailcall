mod http_filter;
mod serde_v8;
mod shim;
mod worker;
pub use std::sync::Arc;
mod sync_v8;

pub use http_filter::HttpFilter;

use crate::{blueprint, HttpIO};

pub fn init_http(http: impl HttpIO, _script: blueprint::Script) -> Arc<dyn HttpIO> {
    // let v8 = SyncV8::new();
    // let script_io = worker::Worker::new(script, &v8, http).unwrap();
    //Arc::new(HttpFilter::new(http, script_io))
    Arc::new(http)
}
