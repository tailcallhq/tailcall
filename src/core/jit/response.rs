use std::borrow::BorrowMut;

use async_graphql_value::ConstValue;
use derive_setters::Setters;
use serde::Serialize;

use super::Positioned;
use crate::core::jit;
use crate::core::merge_right::MergeRight;

#[derive(Clone, Setters, Serialize)]
pub struct Response<Value, Error> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Positioned<Error>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<(String, Value)>,
}

impl<V,E> Default for Response<V,E> {
    fn default() -> Self {
        Self { data: Default::default(), errors: Default::default(), extensions: Default::default() }
    }
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

    pub fn add_errors(&mut self, new_errors: Vec<Positioned<Error>>) {
        self.errors.extend(new_errors);
    }
}

impl Response<ConstValue, jit::Error> {
    pub fn merge_with_async_response(mut self, other: async_graphql::Response) -> Self {
        if let async_graphql::Value::Object(other_obj) = other.data {
            if let Some(async_graphql::Value::Object(self_obj)) = self.data.as_mut() {
                self_obj.extend(other_obj);
            } else {
                self.data = Some(async_graphql::Value::Object(other_obj))
            }
        }
        self
    }
}

impl MergeRight for async_graphql::Response {
    fn merge_right(mut self, other: Self) -> Self {
        if let async_graphql::Value::Object(mut other_obj) = other.data {
            if let async_graphql::Value::Object(self_obj) = std::mem::take(self.data.borrow_mut()) {
                other_obj.extend(self_obj);
                self.data = async_graphql::Value::Object(other_obj);
            }
        }

        self.errors.extend(other.errors);
        self.extensions.extend(other.extensions);

        self
    }
}

impl Response<async_graphql::Value, jit::Error> {
    pub fn into_async_graphql(self) -> async_graphql::Response {
        let mut resp = async_graphql::Response::new(self.data.unwrap_or_default());
        for (name, value) in self.extensions {
            resp = resp.extension(name, value);
        }
        for error in self.errors {
            resp.errors.push(error.into());
        }
        resp
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

    use super::Response;
    use crate::core::jit::{self, Pos, Positioned};
    use crate::core::merge_right::MergeRight;

    #[test]
    fn test_with_response() {
        let value = ConstValue::String("Tailcall - Modern GraphQL Runtime".into());
        let response = Response::<ConstValue, jit::Error>::new(Ok(value.clone()));

        assert!(response.data.is_some());
        assert_eq!(response.data, Some(value));
        assert!(response.errors.is_empty());
        assert!(response.extensions.is_empty());
    }

    #[test]
    fn test_with_error() {
        let error = Positioned::new(
            jit::Error::Validation(jit::ValidationError::ValueRequired),
            Pos { line: 1, column: 2 },
        );
        let response = Response::<ConstValue, jit::Error>::new(Err(error.clone()));

        assert!(response.data.is_none());
        assert!(response.extensions.is_empty());

        assert_eq!(response.errors.len(), 1);
        insta::assert_debug_snapshot!(response.into_async_graphql());
    }

    #[test]
    fn test_adding_errors() {
        let value = ConstValue::String("Tailcall - Modern GraphQL Runtime".into());
        let mut response = Response::<ConstValue, jit::Error>::new(Ok(value.clone()));

        // Initially no errors
        assert!(response.errors.is_empty());

        // Add an error
        let error = Positioned::new(
            jit::Error::Validation(jit::ValidationError::ValueRequired),
            Pos { line: 1, column: 2 },
        );
        response.add_errors(vec![error.clone()]);

        assert_eq!(response.errors.len(), 1);
        insta::assert_debug_snapshot!(response.into_async_graphql());
    }

    #[test]
    fn test_conversion_to_async_graphql() {
        let error1 = Positioned::new(
            jit::Error::Validation(jit::ValidationError::ValueRequired),
            Pos { line: 1, column: 2 },
        );
        let error2 = Positioned::new(
            jit::Error::Validation(jit::ValidationError::EnumInvalid {
                type_of: "EnumDef".to_string(),
            }),
            Pos { line: 3, column: 4 },
        );

        let mut response = Response::<ConstValue, jit::Error>::new(Ok(ConstValue::Null));
        response.add_errors(vec![error2, error1]);

        let async_response = response.into_async_graphql();

        assert_eq!(async_response.errors.len(), 2);
        insta::assert_debug_snapshot!(async_response);
    }

    #[test]
    pub fn test_merging_of_responses() {
        let introspection_response = r#"
        {
            "__type": {
                "name": "User",
                "fields": [
                    {
                        "name": "birthday",
                        "type": {
                            "name": "Date"
                        }
                    },
                    {
                        "name": "id",
                        "type": {
                            "name": "String"
                        }
                    }
                ]
            }
        }
        "#;
        let introspection_data =
            ConstValue::from_json(serde_json::from_str(introspection_response).unwrap()).unwrap();
        let introspection_response = async_graphql::Response::new(introspection_data);

        let user_response = r#"
        {
            "me": {
                "id": 1,
                "name": "John Smith",
                "birthday": "2023-03-08T12:45:26-05:00"
            }
        }
        "#;
        let user_data =
            ConstValue::from_json(serde_json::from_str(user_response).unwrap()).unwrap();
        let query_response = async_graphql::Response::new(user_data);

        let merged_response = introspection_response.merge_right(query_response);

        insta::assert_json_snapshot!(merged_response);
    }

    #[test]
    pub fn test_merging_of_errors() {
        let mut resp1 = async_graphql::Response::new(ConstValue::default());
        let mut err1 = vec![async_graphql::ServerError::new("Error-1", None)];
        resp1.errors.append(&mut err1);

        let mut resp2 = async_graphql::Response::new(ConstValue::default());
        let mut err2 = vec![async_graphql::ServerError::new(
            "Error-2",
            Some(async_graphql::Pos::default()),
        )];
        resp2.errors.append(&mut err2);

        let merged_resp = resp1.merge_right(resp2);
        insta::assert_json_snapshot!(merged_resp);
    }
}
