use std::fs;
use crate::core::blueprint::Blueprint;
use crate::core::http::API_URL_PREFIX;
use crate::core::rest::{EndpointSet, Unchecked};
use crate::core::print_schema;
use crate::cli::fmt::Fmt;
use lazy_static::lazy_static;

pub const FILE_NAME: &str = ".tailcallrc.graphql";
pub const YML_FILE_NAME: &str = ".graphqlrc.yml";
pub const JSON_FILE_NAME: &str = ".tailcallrc.schema.json";

lazy_static! {
    pub static ref TRACKER: tailcall_tracker::Tracker = tailcall_tracker::Tracker::default();
}

/// Checks if file or folder already exists or not.
pub(super) fn is_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub(super) fn log_endpoint_set(endpoint_set: &EndpointSet<Unchecked>) {
    let mut endpoints = endpoint_set.get_endpoints().clone();
    endpoints.sort_by(|a, b| {
        let method_a = a.get_method();
        let method_b = b.get_method();
        if method_a.eq(method_b) {
            a.get_path().as_str().cmp(b.get_path().as_str())
        } else {
            method_a.to_string().cmp(&method_b.to_string())
        }
    });
    for endpoint in endpoints {
        tracing::info!(
            "Endpoint: {} {}{} ... ok",
            endpoint.get_method(),
            API_URL_PREFIX,
            endpoint.get_path().as_str()
        );
    }
}

pub(super) fn display_schema(blueprint: &Blueprint) {
    Fmt::display(Fmt::heading("GraphQL Schema:\n"));
    let sdl = blueprint.to_schema();
    Fmt::display(format!("{}\n", print_schema::print_schema(sdl)));
}

