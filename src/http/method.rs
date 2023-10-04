use serde::{Deserialize, Serialize};
use enum_definition_derive::EnumDefinition;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[derive(EnumDefinition)]
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
  use crate::http::method::Method;
  use crate::document::print;
  use async_graphql::parser::types::{ServiceDocument, TypeSystemDefinition, TypeDefinition};
  use async_graphql::{Pos, Positioned};

  #[test]
  fn test_enum_definition() {
    let enum_def: TypeDefinition = Method::enum_definition();
    let s = ServiceDocument {
        definitions: vec![TypeSystemDefinition::Type(
            Positioned::new(enum_def, Pos::default())
        )]
    };
    let pd = print(s);
    println!("{}", pd);

    assert!(true);

  }
}
