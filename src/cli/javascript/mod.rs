use std::collections::BTreeMap;
use std::sync::Arc;

use hyper::header::{HeaderName, HeaderValue};

pub mod codec;

mod runtime;

pub use runtime::Runtime;

use crate::core::{blueprint, WorkerIO};

pub fn init_worker_io<T, V>(script: blueprint::Script) -> Arc<dyn WorkerIO<T, V> + Send + Sync>
where
    Runtime: WorkerIO<T, V>,
{
    (Arc::new(Runtime::new(script))) as _
}

fn create_header_map(
    headers: BTreeMap<String, String>,
) -> anyhow::Result<reqwest::header::HeaderMap> {
    let mut header_map = reqwest::header::HeaderMap::new();
    for (key, value) in headers.iter() {
        let key = HeaderName::from_bytes(key.as_bytes())?;
        let value = HeaderValue::from_str(value.as_str())?;
        header_map.insert(key, value);
    }
    Ok(header_map)
}
