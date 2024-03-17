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
    use async_graphql::{Name, Pos, Positioned};
    use indexmap::IndexMap;

    use super::*;

    // Helper function to create Positioned<Value> easily
    fn pos<A>(a: A) -> Positioned<A> {
        Positioned::new(a, Pos::default())
    }

    #[test]
    fn test_rest_try_from_empty_directive() {
        let directive = Directive { name: pos(Name::new("rest")), arguments: vec![] };
        let rest = Rest::try_from(&directive).unwrap();
        assert_eq!(rest.path, "");
        assert_eq!(rest.method, None);
        assert!(rest.query.is_empty());
        assert_eq!(rest.body, None);
        assert_eq!(rest, Rest::default());
    }

    #[test]
    fn test_rest_try_from_directive_with_invalid_method() {
        let directive = Directive {
            name: pos(Name::new("rest")),
            arguments: vec![(
                pos(Name::new("method")),
                pos(Value::String("invalid".to_string())),
            )],
        };

        let rest = Rest::try_from(&directive);

        assert!(rest.is_err());
    }

    fn test_rest_try_from_directive_from_method(method: Method) {
        let query: [(Name, Value); 3] = [
            (Name::new("b"), Value::Variable(Name::new("b"))),
            (Name::new("c"), Value::Variable(Name::new("c"))),
            (Name::new("d"), Value::Variable(Name::new("d"))),
        ];

        let expected = Rest::default()
            .method(Some(method.clone()))
            .body(Some("v".to_string()))
            .path("/foo/$a".to_string())
            .query(
                query
                    .iter()
                    .map(|(k, v)| {
                        assert!(
                            matches!(v, Value::Variable(_)),
                            "Expected Value::Variable, got {:?}",
                            v
                        );
                        let Value::Variable(v) = v.clone() else {
                            unreachable!("Value::Variable was asserted above");
                        };

                        (k.to_string(), v.to_string())
                    })
                    .collect(),
            );

        let directive = Directive {
            name: pos(Name::new("rest")),
            arguments: vec![
                (
                    pos(Name::new("method")),
                    pos(Value::Enum(Name::new(method.to_hyper().as_str()))),
                ),
                (
                    pos(Name::new("path")),
                    pos(Value::String(expected.path.clone())),
                ),
                (
                    pos(Name::new("query")),
                    pos(Value::Object(IndexMap::from(query))),
                ),
                (
                    pos(Name::new("body")),
                    pos(Value::Variable(Name::new(expected.body.clone().unwrap()))),
                ),
            ],
        };

        let rest = Rest::try_from(&directive).unwrap();
        assert_eq!(rest, expected);
    }

    #[test]
    fn test_rest_try_from_directive() {
        for method in [
            Method::CONNECT,
            Method::DELETE,
            Method::GET,
            Method::HEAD,
            Method::OPTIONS,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ] {
            test_rest_try_from_directive_from_method(method);
        }
    }
}
