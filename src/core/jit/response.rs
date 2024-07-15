use async_graphql::Positioned;
use derive_setters::Setters;
use serde::Serialize;

use crate::core::jit;

#[derive(Setters, Serialize)]
pub struct Response<Value, Error> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Positioned<Error>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<(String, Value)>,
}

impl<Value, Error> Response<Value, Error> {
    pub fn new(result: Result<Value, Positioned<Error>>) -> Self {
        match result {
            Ok(value) => Response {
                data: Some(value),
                errors: Vec::new(),
                extensions: Vec::new(),
            },
            Err(error) => Response { data: None, errors: vec![error], extensions: Vec::new() },
        }
    }
}

impl Response<async_graphql::Value, jit::Error> {
    pub fn into_async_graphql(self) -> async_graphql::Response {
        let mut resp = async_graphql::Response::new(self.data.unwrap_or_default());
        for (name, value) in self.extensions {
            resp = resp.extension(name, value);
        }
        for error in self.errors {
            resp.errors.push(async_graphql::ServerError::new(
                error.node.to_string(),
                Some(error.pos),
            ));
        }
        resp
    }
}
