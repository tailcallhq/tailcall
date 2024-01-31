mod async_wrapper;
mod serde_v8;
mod shim;
mod worker;


use std::sync::Arc;
use crate::{blueprint, HttpIO};

pub fn init_http(http: impl HttpIO, script: blueprint::Script) -> Arc<dyn HttpIO> {
    let async_js_wrapper = async_wrapper::JsTokioWrapper::new(script, http);

    Arc::new(async_js_wrapper)
}
