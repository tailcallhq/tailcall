use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Default,
    schemars::JsonSchema,
    Display,
)]
pub enum Method {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl Method {
    pub fn to_hyper(self) -> http::Method {
        match self {
            Method::GET => http::Method::GET,
            Method::POST => http::Method::POST,
            Method::PUT => http::Method::PUT,
            Method::PATCH => http::Method::PATCH,
            Method::DELETE => http::Method::DELETE,
            Method::HEAD => http::Method::HEAD,
            Method::OPTIONS => http::Method::OPTIONS,
            Method::CONNECT => http::Method::CONNECT,
            Method::TRACE => http::Method::TRACE,
        }
    }
}
