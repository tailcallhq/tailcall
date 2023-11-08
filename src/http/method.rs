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

impl Method {
  pub fn as_str(&self) -> &str {
    match self {
      Method::GET => "GET",
      Method::POST => "POST",
      Method::PUT => "PUT",
      Method::PATCH => "PATCH",
      Method::DELETE => "DELETE",
      Method::HEAD => "HEAD",
      Method::OPTIONS => "OPTIONS",
      Method::CONNECT => "CONNECT",
      Method::TRACE => "TRACE",
    }
  }
}

impl From<Method> for reqwest::Method {
  fn from(method: Method) -> Self {
    (&method).into()
  }
}

impl From<&Method> for reqwest::Method {
  fn from(method: &Method) -> Self {
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
