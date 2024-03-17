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
    }

    #[test]
    fn test_rest_try_from_directive() {
        // let directive = Directive { name: pos(Name::new("rest")) };
        let query: [(Name, Value); 3] = [
            (Name::new("b"), Value::Variable(Name::new("b"))),
            (Name::new("c"), Value::Variable(Name::new("c"))),
            (Name::new("d"), Value::Variable(Name::new("d"))),
        ];
        let path = "/foo/$a".to_string();
        let body = "v".to_string();
        let directive = Directive {
            name: pos(Name::new("rest")),
            arguments: vec![
                (
                    pos(Name::new("method")),
                    pos(Value::Enum(Name::new("POST"))),
                ),
                (
                    pos(Name::new("path")),
                    pos(Value::String(path.clone())),
                ),
                (
                    pos(Name::new("query")),
                    pos(Value::Object(IndexMap::from(query.clone()))),
                ),
                (
                    pos(Name::new("body")),
                    pos(Value::Variable(Name::new(body))),
                ),
            ],
        };

        let rest = Rest::try_from(&directive).unwrap();
        assert_eq!(rest.path, path);
        assert_eq!(rest.method.unwrap(), Method::POST);
        assert!(!rest.query.is_empty());
        assert_eq!(
            rest.query,
            query
                .iter()
                .map(|(k, v)| {
                    let Value::Variable(v) = v.clone() else {
                        assert!(matches!(v, Value::Variable(_)), "Expected Value::Variable, got {:?}", v);
                    };

                    (k.to_string(), v.to_string())
                })
                .collect()
        );
        assert_eq!(rest.body.unwrap(), "v");
    }
}
