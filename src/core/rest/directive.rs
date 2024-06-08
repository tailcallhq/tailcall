use std::collections::BTreeMap;

use async_graphql::parser::types::Directive;
use async_graphql_value::Value;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use super::{Error, Result};
use crate::core::http::Method;
use crate::core::is_default;

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
    type Error = Error;

    fn try_from(directive: &Directive) -> Result<Self> {
        let mut rest = Rest::default();

        let mut has_path = false;
        let mut has_method = false;

        for (k, v) in directive.arguments.iter() {
            match k.node.as_str() {
                "path" => {
                    rest.path = serde_json::from_str(v.node.to_string().as_str())?;
                    has_path = true;
                }
                "method" => {
                    let value = serde_json::Value::String(v.node.to_string().to_uppercase());
                    rest.method = serde_json::from_value(value)?;
                    has_method = true;
                }
                "query" => {
                    if let Value::Object(map) = &v.node {
                        map.iter()
                            .filter_map(|(k, v)| {
                                if let Value::Variable(v) = v {
                                    Some((k.as_str().to_owned(), v.as_str().to_string()))
                                } else {
                                    None
                                }
                            })
                            .for_each(|(k, v)| {
                                rest.query.insert(k, v);
                            })
                    }
                }
                "body" => {
                    if let Value::Variable(v) = &v.node {
                        rest.body = Some(v.to_string());
                    }
                }
                _ => {}
            };
        }

        match (has_method, has_path) {
            (true, true) => Ok(rest),
            (true, false) => Err(Error::Missing {
                msg: "Path not provided in the directive".to_string(),
                directive: directive.clone(),
            }),
            (false, true) => Err(Error::Missing {
                msg: "Method not provided in the directive".to_string(),
                directive: directive.clone(),
            }),
            (false, false) => Err(Error::Missing {
                msg: "Method and Path not provided in the directive".to_string(),
                directive: directive.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_graphql::parser::types::Directive;
    use once_cell::sync::Lazy;

    use super::*;

    const DEFAULT_QUERY_PARAM: &str = "$a: Int, $v: String";
    static DEFAULT_REST_QUERY: Lazy<RestQueryParam> =
        Lazy::new(|| RestQueryParam::new("/foo/$a", "$v"));

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

    fn generate_query_with_directive(rest_directive: &str, query_parameter: &str) -> String {
        format!("query ({query_parameter}) @rest({rest_directive}) {{ value }}")
    }

    #[derive(Clone)]
    struct RestQueryParam {
        path: String,
        body: String,
    }

    impl RestQueryParam {
        fn new(path: &str, body: &str) -> Self {
            Self { path: path.into(), body: body.into() }
        }

        fn string_with_method(&self, method: &str) -> String {
            format!(
                "method: {}, path: \"{}\", body: {}",
                method, self.path, self.body
            )
        }

        fn string_without_method(&self) -> String {
            format!("path: \"{}\", body: {}", self.path, self.body)
        }

        fn string_without_path(&self, method: &str) -> String {
            format!("method: {}, body: {}", method, self.body)
        }

        fn string_without_path_default_method(&self) -> String {
            format!("method: GET, body: {}", self.body)
        }
    }

    fn generate_method_variant(
        query: &RestQueryParam,
        method: Method,
        default_query_param: &str,
    ) -> (String, Rest) {
        (
            generate_query_with_directive(
                &query.string_with_method(&method.to_string()),
                default_query_param,
            ),
            default_rest_with("/foo/$a", method, "v"),
        )
    }

    fn all_methods_valid() -> HashMap<String, Rest> {
        let default_rest_query = DEFAULT_REST_QUERY.clone();
        HashMap::from([
            generate_method_variant(&default_rest_query, Method::GET, DEFAULT_QUERY_PARAM),
            generate_method_variant(&default_rest_query, Method::PUT, DEFAULT_QUERY_PARAM),
            generate_method_variant(&default_rest_query, Method::DELETE, DEFAULT_QUERY_PARAM),
            generate_method_variant(&default_rest_query, Method::HEAD, DEFAULT_QUERY_PARAM),
            generate_method_variant(&default_rest_query, Method::PATCH, DEFAULT_QUERY_PARAM),
        ])
    }

    #[test]
    fn test_directive_to_rest_methods() {
        let (actual, expected): (Vec<_>, Vec<_>) = all_methods_valid()
            .into_iter()
            .map(|(query, expected_rest)| {
                let directive = query_to_directive(&query);
                let actual = Rest::try_from(&directive).unwrap();
                (actual, expected_rest)
            })
            .unzip();

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test]
    fn test_directive_to_rest_should_fail() {
        let default_rest_query = DEFAULT_REST_QUERY.clone();
        let directives = [
            default_rest_query.string_without_path("GET"),
            default_rest_query.string_without_path("PUT"),
            default_rest_query.string_without_path("DELETE"),
            default_rest_query.string_without_path("UPDATE"),
            default_rest_query.string_without_method(),
        ]
        .iter()
        .map(|query| generate_query_with_directive(query, DEFAULT_QUERY_PARAM))
        .map(|query| query_to_directive(&query))
        .map(|directive| Rest::try_from(&directive))
        .map(|result| result.is_err())
        .collect::<Vec<_>>();

        pretty_assertions::assert_eq!(directives, vec![true; 5]);
    }

    #[test]
    #[should_panic(expected = "Path not provided in the directive")]
    fn directive_to_rest_without_path() {
        let default_rest_query = DEFAULT_REST_QUERY.clone();
        let query = default_rest_query.string_without_path_default_method();
        let query = generate_query_with_directive(&query, DEFAULT_QUERY_PARAM);
        let directive = query_to_directive(&query);
        // Will panic
        Rest::try_from(&directive).unwrap();
    }

    #[test]
    #[should_panic(expected = "Method not provided in the directive")]
    fn directive_to_rest_without_method() {
        let default_rest_query = DEFAULT_REST_QUERY.clone();
        let query = default_rest_query.string_without_method();
        let query = generate_query_with_directive(&query, DEFAULT_QUERY_PARAM);
        let directive = query_to_directive(&query);
        // Will panic
        Rest::try_from(&directive).unwrap();
    }
}
