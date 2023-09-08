use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
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

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> Self {
        match method {
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
