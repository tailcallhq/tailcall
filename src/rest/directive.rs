use std::collections::BTreeMap;

use async_graphql::parser::types::Directive;
use async_graphql_value::Value;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::http::Method;
use crate::is_default;

/// A structure that represents the REST directive.
/// It allows easy parsing of the GraphQL query and extracting the REST
/// directive.
#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Setters)]
pub(crate) struct Rest {
    pub path: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub method: Option<Method>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub query: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub body: Option<String>,
}

impl TryFrom<&Directive> for Rest {
    type Error = anyhow::Error;

    fn try_from(directive: &Directive) -> anyhow::Result<Self> {
        let mut rest = Rest::default();

        for (k, v) in directive.arguments.iter() {
            if k.node.as_str() == "path" {
                rest.path = serde_json::from_str(v.node.to_string().as_str())?;
            }
            if k.node.as_str() == "method" {
                let value = serde_json::Value::String(v.node.to_string().to_uppercase());
                rest.method = serde_json::from_value(value)?;
            }
            if k.node.as_str() == "query" {
                if let Value::Object(map) = &v.node {
                    for (k, v) in map {
                        if let Value::Variable(v) = v {
                            rest.query
                                .insert(k.as_str().to_owned(), v.as_str().to_string());
                        }
                    }
                }
            }
            if k.node.as_str() == "body" {
                if let Value::Variable(v) = &v.node {
                    rest.body = Some(v.to_string());
                }
            }
        }

        Ok(rest)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_graphql::parser::types::Directive;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use super::*;

    fn query_to_directive(query: &str) -> Directive {
        async_graphql::parser::parse_query(query)
            .unwrap()
            .operations
            .iter()
            .next()
            .unwrap()
            .1
            .node
            .directives
            .first()
            .unwrap()
            .node
            .clone()
    }

    fn default_rest_with(path: &str, method: Method, body: &str) -> Rest {
        Rest::default()
            .path(path.to_string())
            .method(Some(method))
            .body(Some(body.to_string()))
    }

    fn valid_all_method_queries() -> HashMap<String, Rest> {
        hashmap! {
            // GET method
            r#"query ($a: Int, $v: String)
                @rest(method: GET, path: "/foo/$a", body: $v) {
                    value
                }"#.to_string() => 
            default_rest_with("/foo/$a", Method::GET, "v"),

            // POST method
            r#"query ($a: Int, $v: String)
                @rest(method: POST, path: "/foo/$a", body: $v) {
                    value
                }"#.to_string() => 
            default_rest_with("/foo/$a", Method::POST, "v"),

            // PUT method
            r#"query ($a: Int, $v: String)
                @rest(method: PUT, path: "/foo/$a", body: $v) {
                    value
                }"#.to_string() =>
            default_rest_with("/foo/$a", Method::PUT, "v"),

            // DELETE method
            r#"query ($a: Int, $v: String)
                @rest(method: DELETE, path: "/foo/$a", body: $v) {
                    value
                }"#.to_string() =>
            default_rest_with("/foo/$a", Method::DELETE, "v"),

            // PATCH method
            r#"query ($a: Int, $v: String)
                @rest(method: PATCH, path: "/foo/$a", body: $v) {
                    value
                }"#.to_string() =>
            default_rest_with("/foo/$a", Method::PATCH, "v"),


            // HEAD method
            r#"query ($a: Int, $v: String)
                @rest(method: HEAD, path: "/foo/$a", body: $v) {
                    value
                }"#.to_string() =>
            default_rest_with("/foo/$a", Method::HEAD, "v"),

            // OPTIONS method
            r#"query ($a: Int, $v: String)
                @rest(method: OPTIONS, path: "/foo/$a", body: $v) {
                    value
                }"#.to_string() =>
            default_rest_with("/foo/$a", Method::OPTIONS, "v"),
        }
    }

    #[test]
    fn test_directive_to_rest_methods() {
        for (query, expected_rest) in valid_all_method_queries() {
            let directive = query_to_directive(&query);
            let actual = Rest::try_from(&directive).unwrap();
            assert_eq!(actual, expected_rest);
        }
    }
}
