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
    pub fn to_hyper(self) -> hyper::Method {
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
    pub fn to_reqwest(self) -> reqwest::Method {
        match self {
            Method::GET => reqwest::Method::GET,
            Method::POST => reqwest::Method::POST,
            Method::PUT => reqwest::Method::PUT,
            Method::PATCH => reqwest::Method::PATCH,
            Method::DELETE => reqwest::Method::DELETE,
            Method::HEAD => reqwest::Method::HEAD,
            Method::OPTIONS => reqwest::Method::OPTIONS,
            Method::CONNECT => reqwest::Method::CONNECT,
            Method::TRACE => reqwest::Method::TRACE,
        }
    }
}
