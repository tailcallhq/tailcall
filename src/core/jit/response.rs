use derive_setters::Setters;
use serde::Serialize;

use crate::core::jit;

#[derive(Setters, Serialize)]
pub struct Response<Value, Error> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Error>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<(String, Value)>,
}

impl<Value, Error> Response<Value, Error> {
    pub fn new(result: Result<Value, Error>) -> Self {
        match result {
            Ok(value) => Response {
                data: Some(value),
                errors: Vec::new(),
                extensions: Vec::new(),
            },
            Err(errors) => Response { data: None, errors: vec![errors], extensions: Vec::new() },
        }
    }
}

impl Response<async_graphql::Value, jit::Error> {
    pub fn try_into_hyper_response(self) -> anyhow::Result<hyper::Response<hyper::Body>> {
        let body = serde_json::to_string(&self)?;
        let resp = hyper::Response::builder()
            .header("Content-Type", "application/json")
            .body(hyper::Body::from(body))?;
        Ok(resp)
    }
}
