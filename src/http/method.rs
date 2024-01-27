use serde::{Deserialize, Serialize};
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, schemars::JsonSchema,
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
    pub fn to_hyper(&self) -> hyper::Method {
        match self {
            Method::GET => hyper::Method::GET,
            Method::POST => hyper::Method::POST,
            Method::PUT => hyper::Method::PUT,
            Method::PATCH => hyper::Method::PATCH,
            Method::DELETE => hyper::Method::DELETE,
            Method::HEAD => hyper::Method::HEAD,
            Method::OPTIONS => hyper::Method::OPTIONS,
            Method::CONNECT => hyper::Method::CONNECT,
            Method::TRACE => hyper::Method::TRACE,
        }
    }
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "CONNECT" => Method::CONNECT,
            "TRACE" => Method::TRACE,
            _ => Method::GET,
        }
    }
}
