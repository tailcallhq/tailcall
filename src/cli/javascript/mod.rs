mod serde_v8;
mod shim;
mod worker;
pub use std::sync::Arc;
mod sync_v8;

use async_std::task::block_on;

use crate::{blueprint, HttpIO};

pub fn init_http(http: impl HttpIO, script: blueprint::Script) -> Arc<dyn HttpIO> {
    let http = block_on(worker::Worker::new(script, http)).unwrap();
    Arc::new(http)
}
