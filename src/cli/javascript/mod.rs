use std::collections::BTreeMap;
pub use std::sync::Arc;

use hyper::header::{HeaderName, HeaderValue};

mod channel;
mod http_filter;
mod external;
mod js_request;
mod js_response;
mod runtime;

pub use http_filter::HttpFilter;
pub use js_request::JsRequest;
pub use js_response::JsResponse;
pub use runtime::Runtime;

use crate::{blueprint, cli::javascript::external::ExternalRuntime, HttpIO};

pub fn init_http(http: impl HttpIO, script: blueprint::Script) -> Arc<dyn HttpIO + Sync + Send> {
    log::debug!("Initializing JavaScript HTTP filter: {}", script.source);
    let script_io = ExternalRuntime::new("http://localhost:3000".to_owned());
    Arc::new(HttpFilter::new(http, script_io))
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
