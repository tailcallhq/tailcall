use std::str::FromStr;

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

impl FromStr for Method {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "PATCH" => Ok(Self::PATCH),
            "DELETE" => Ok(Self::DELETE),
            "HEAD" => Ok(Self::HEAD),
            "OPTIONS" => Ok(Self::OPTIONS),
            "CONNECT" => Ok(Self::CONNECT),
            "TRACE" => Ok(Self::TRACE),
            _ => Err("Wrong method name"),
        }
    }
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
}
