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

impl TryFrom<hyper::Method> for Method {
    type Error = anyhow::Error;

    fn try_from(method: hyper::Method) -> anyhow::Result<Self> {
        Ok(match method {
            hyper::Method::GET => Method::GET,
            hyper::Method::POST => Method::POST,
            hyper::Method::PUT => Method::PUT,
            hyper::Method::PATCH => Method::PATCH,
            hyper::Method::DELETE => Method::DELETE,
            hyper::Method::HEAD => Method::HEAD,
            hyper::Method::OPTIONS => Method::OPTIONS,
            hyper::Method::CONNECT => Method::CONNECT,
            hyper::Method::TRACE => Method::TRACE,
            _ => Err(anyhow::anyhow!("Unsupported HTTP method"))?,
        })
    }
}