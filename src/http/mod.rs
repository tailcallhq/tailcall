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

pub use cache::*;
pub use data_loader::*;
pub use data_loader_request::*;
use derive_setters::Setters;
pub use method::Method;
pub use request_context::RequestContext;
pub use request_handler::{graphiql, handle_request, API_URL_PREFIX};
pub use request_template::RequestTemplate;
pub use response::*;

pub use crate::app_context::AppContext;

#[derive(Setters, Default, Clone)]
pub struct HttpFilter {
  pub on_request: Option<String>
}