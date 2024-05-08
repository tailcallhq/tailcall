#[cfg(feature = "js")]
mod enable_js;

use std::sync::Arc;

#[cfg(feature = "js")]
pub use enable_js::*;

#[cfg(not(feature = "js"))]
mod runtime_no_js;
#[cfg(not(feature = "js"))]
pub use runtime_no_js::*;

#[derive(Debug)]
pub struct JsResponse(Response<String>);
#[derive(Debug)]
pub struct JsRequest(reqwest::Request);

#[derive(Debug)]
pub enum Event {
    Request(JsRequest),
}

#[derive(Debug)]
pub enum Command {
    Request(JsRequest),
    Response(JsResponse),
}

use crate::blueprint::Script;
use crate::http::Response;

pub fn init_rt(script: Option<crate::blueprint::Script>) -> Arc<Runtime> {
    if let Some(script) = script {
        Arc::new(Runtime::new(script))
    } else {
        Arc::new(Runtime::new(Script {
            source: "".to_string(),
            timeout: None,
        })) // TODO
    }
}
