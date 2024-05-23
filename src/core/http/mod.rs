pub use cache::*;
pub use data_loader::*;
pub use data_loader_request::*;
use headers::HeaderValue;
pub use method::Method;
pub use request_context::RequestContext;
pub use request_handler::{handle_request, API_URL_PREFIX};
pub use request_template::RequestTemplate;
pub use response::*;
pub use request::*;

pub use crate::core::app_context::AppContext;

mod data_loader;

mod cache;
mod data_loader_request;
mod method;
mod request_context;
mod request_handler;
mod request_template;
mod response;
pub mod showcase;
mod telemetry;
mod request;

pub static TAILCALL_HTTPS_ORIGIN: HeaderValue = HeaderValue::from_static("https://tailcall.run");
pub static TAILCALL_HTTP_ORIGIN: HeaderValue = HeaderValue::from_static("http://tailcall.run");
