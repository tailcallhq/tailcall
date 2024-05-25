use std::collections::BTreeMap;
use std::sync::Arc;

use hyper::header::{HeaderName, HeaderValue};

mod js_request;

mod js_response;

pub mod request_filter;

mod runtime;

pub use request_filter::RequestFilter;
pub use runtime::Runtime;

use crate::core::{blueprint, HttpIO, WorkerIO};

pub fn init_http(
    http: Arc<impl HttpIO>,
    script: blueprint::Script,
) -> Arc<dyn HttpIO + Sync + Send> {
    tracing::debug!("Initializing JavaScript HTTP filter: {}", script.source);
    let script_io = Arc::new(Runtime::new(script));
    Arc::new(RequestFilter::new(http, script_io))
}

pub fn init_worker_io<T, V>(script: blueprint::Script) -> Arc<dyn WorkerIO<T, V> + Send + Sync>
where
    Runtime: WorkerIO<T, V>,
{
    (Arc::new(Runtime::new(script))) as _
}

fn create_header_map(
    headers: BTreeMap<String, String>,
) -> anyhow::Result<hyper::header::HeaderMap> {
    let mut header_map = hyper::header::HeaderMap::new();
    for (key, value) in headers.iter() {
        let key = HeaderName::from_bytes(key.as_bytes())?;
        let value = HeaderValue::from_str(value.as_str())?;
        header_map.insert(key, value);
    }
    Ok(header_map)
}
