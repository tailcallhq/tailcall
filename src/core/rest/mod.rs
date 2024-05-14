mod directive;
mod endpoint;
mod endpoint_set;
mod operation;
mod partial_request;
mod path;
mod query_params;
mod type_map;
mod typed_variables;

use bytes::Bytes;
pub use endpoint_set::{Checked, EndpointSet, Unchecked};
use http_body_util::Full;

type Request = hyper::Request<Full<Bytes>>;
