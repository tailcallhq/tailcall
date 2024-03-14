mod directive;
mod endpoint;
mod endpoint_set;
mod operation;
mod partial_request;
mod path;
mod query_params;
mod type_map;
mod typed_variables;

pub use endpoint_set::EndpointSet;

type Request = hyper::Request<hyper::Body>;
