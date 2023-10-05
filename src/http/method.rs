use enum_definition_derive::EnumDefinition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, EnumDefinition)]
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

#[cfg(test)]

mod tests {
  use async_graphql::parser::types::{ServiceDocument, TypeDefinition, TypeSystemDefinition};
  use async_graphql::{Pos, Positioned};

  use crate::document::print;
  use crate::http::method::Method;

  #[test]
  fn test_enum_definition() {
    let enum_def: TypeDefinition = Method::enum_definition();
    let service_doc =
      ServiceDocument { definitions: vec![TypeSystemDefinition::Type(Positioned::new(enum_def, Pos::default()))] };
    let actual = print(service_doc);
    let expected = "enum Method {
  GET
  POST
  PUT
  PATCH
  DELETE
  HEAD
  OPTIONS
  CONNECT
  TRACE
}";
    println!("{}", actual);
    assert_eq!(actual, expected);
  }
}
