use std::collections::BTreeMap;

use async_graphql::parser::types::Type;
use async_graphql::{Name, Variables};
use async_graphql_value::ConstValue;
use derive_setters::Setters;
use rest::Rest;
use typed_variable::{TypedVariable, UrlParamType};

use self::query_params::QueryParams;
use crate::async_graphql_hyper::GraphQLRequest;
use crate::directive::DirectiveCodec;
use crate::document::print_operation;
use crate::http::Method;

type Request = hyper::Request<hyper::Body>;

#[derive(Clone, Debug, PartialEq)]
enum Segment {
    Literal(String),
    Param(TypedVariable),
}

impl Segment {
    pub fn lit(s: &str) -> Self {
        Self::Literal(s.to_string())
    }

    pub fn param(t: UrlParamType, s: &str) -> Self {
        Self::Param(TypedVariable::new(t, s))
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Path {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct TypeMap(BTreeMap<String, Type>);

impl TypeMap {
    fn get(&self, key: &str) -> Option<&Type> {
        self.0.get(key)
    }
}

impl From<Vec<(&str, Type)>> for TypeMap {
    fn from(map: Vec<(&str, Type)>) -> Self {
        Self(map.iter().map(|a| (a.0.to_owned(), a.1.clone())).collect())
    }
}

impl Path {
    fn parse(q: &TypeMap, input: &str) -> anyhow::Result<Self> {
        let variables = q;

        let mut segments = Vec::new();
        for s in input.split('/').filter(|s| !s.is_empty()) {
            if let Some(key) = s.strip_prefix('$') {
                let value = variables.get(key).ok_or(anyhow::anyhow!(
                    "undefined param: {} in {}",
                    s,
                    input
                ))?;
                let t = UrlParamType::try_from(value)?;
                segments.push(Segment::param(t, key));
            } else {
                segments.push(Segment::lit(s));
            }
        }
        Ok(Self { segments })
    }

    fn matches(&self, path: &str) -> Option<Variables> {
        let mut variables = Variables::default();
        let mut req_segments = path.split('/').filter(|s| !s.is_empty());
        for (segment, req_segment) in self.segments.iter().zip(&mut req_segments) {
            match segment {
                Segment::Literal(segment) => {
                    if segment != req_segment {
                        return None;
                    }
                }
                Segment::Param(t_var) => {
                    let tpe = t_var.to_value(req_segment).ok()?;
                    variables.insert(Name::new(t_var.name.clone()), tpe);
                }
            }
        }

        // If there is still some segments in incoming request it should not match
        if req_segments.next().is_some() {
            return None;
        }

        Some(variables)
    }
}

mod typed_variable {

    use async_graphql::parser::types::{BaseType, Type};
    use async_graphql_value::ConstValue;
    use derive_setters::Setters;

    #[derive(Clone, Debug, PartialEq)]
    pub enum UrlParamType {
        String,
        Number(N),
        Boolean,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum N {
        Int,
        Float,
    }

    impl N {
        fn to_value(&self, value: &str) -> anyhow::Result<ConstValue> {
            Ok(match self {
                Self::Int => ConstValue::from(value.parse::<i64>()?),
                Self::Float => ConstValue::from(value.parse::<f64>()?),
            })
        }
    }

    impl UrlParamType {
        fn to_value(&self, value: &str) -> anyhow::Result<ConstValue> {
            Ok(match self {
                Self::String => ConstValue::String(value.to_string()),
                Self::Number(n) => n.to_value(value)?,
                Self::Boolean => ConstValue::Boolean(value.parse()?),
            })
        }
    }

    impl TryFrom<&Type> for UrlParamType {
        type Error = anyhow::Error;
        fn try_from(value: &Type) -> anyhow::Result<Self> {
            match &value.base {
                BaseType::Named(name) => match name.as_str() {
                    "String" => Ok(Self::String),
                    "Int" => Ok(Self::Number(N::Int)),
                    "Boolean" => Ok(Self::Boolean),
                    "Float" => Ok(Self::Number(N::Float)),
                    _ => Err(anyhow::anyhow!("unsupported type: {}", name)),
                },
                // TODO: support for list types
                _ => Err(anyhow::anyhow!("unsupported type: {:?}", value)),
            }
        }
    }
    #[derive(Clone, Debug, PartialEq, Setters)]
    pub struct TypedVariable {
        pub type_of: UrlParamType,
        pub name: String,
        // TODO: validate types for query
        pub nullable: bool,
    }

    impl TypedVariable {
        pub fn new(tpe: UrlParamType, name: &str) -> Self {
            Self { type_of: tpe, name: name.to_string(), nullable: false }
        }

        pub fn to_value(&self, value: &str) -> anyhow::Result<ConstValue> {
            self.type_of.to_value(value)
        }
    }
}

mod query_params {

    use std::collections::BTreeMap;

    use async_graphql::{Name, Variables};

    use super::typed_variable::{TypedVariable, UrlParamType};
    use super::TypeMap;

    #[derive(Debug, PartialEq, Default, Clone)]
    pub struct QueryParams {
        params: Vec<(String, TypedVariable)>,
    }

    impl From<Vec<(&str, TypedVariable)>> for QueryParams {
        fn from(value: Vec<(&str, TypedVariable)>) -> Self {
            Self {
                params: value.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            }
        }
    }

    impl QueryParams {
        pub fn try_from_map(q: &TypeMap, map: BTreeMap<String, String>) -> anyhow::Result<Self> {
            let mut params = Vec::new();
            for (k, v) in map {
                let t = UrlParamType::try_from(
                    q.get(&k)
                        .ok_or(anyhow::anyhow!("undefined query param: {}", k))?,
                )?;
                params.push((k, TypedVariable::new(t, &v)));
            }
            Ok(Self { params })
        }

        pub fn matches(&self, query_params: BTreeMap<String, String>) -> Option<Variables> {
            let mut variables = Variables::default();
            for (key, t_var) in &self.params {
                if let Some(query_param) = query_params.get(key) {
                    let value = t_var.to_value(query_param).ok()?;
                    variables.insert(Name::new(t_var.name.clone()), value);
                }
            }
            Some(variables)
        }
    }
}

#[derive(Debug, Setters, Clone)]
pub struct Endpoint {
    method: Method,
    path: Path,

    // Can use persisted queries for better performance
    query_params: QueryParams,
    body: Option<String>,
    graphql_query: String,
}

mod rest {
    use std::collections::BTreeMap;

    use async_graphql::parser::types::Directive;
    use async_graphql_value::Value;
    use derive_setters::Setters;
    use serde::{Deserialize, Serialize};

    use crate::http::Method;
    use crate::is_default;

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
                    rest.method = serde_json::from_str(v.node.to_string().to_uppercase().as_str())?;
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
}

/// Creates a Rest instance from @rest directive

impl Endpoint {
    pub fn try_new(operations: &str) -> anyhow::Result<Vec<Self>> {
        let doc = async_graphql::parser::parse_query(operations)?;
        let mut endpoints = Vec::new();

        for (_, op) in doc.operations.iter() {
            let type_map = TypeMap(
                op.node
                    .variable_definitions
                    .iter()
                    .map(|pos| {
                        (
                            pos.node.name.node.to_string(),
                            pos.node.var_type.node.clone(),
                        )
                    })
                    .collect::<BTreeMap<_, _>>(),
            );

            let rest = op.node.directives.iter().find_map(|d| {
                if d.node.name.node == Rest::directive_name() {
                    let rest = Rest::try_from(&d.node);
                    Some(rest)
                } else {
                    None
                }
            });

            let graphql_query = print_operation(&op.node);

            if let Some(rest) = rest {
                let rest = rest?;
                let endpoint = Self {
                    method: rest.method.unwrap_or_default(),
                    path: Path::parse(&type_map, &rest.path)?,
                    query_params: QueryParams::try_from_map(&type_map, rest.query)?,
                    body: rest.body,
                    graphql_query,
                };
                endpoints.push(endpoint);
            }
        }

        Ok(endpoints)
    }

    pub fn matches<'a>(&'a self, request: &Request) -> Option<PartialRequest<'a>> {
        let query_params = request
            .uri()
            .query()
            .map(|query| serde_urlencoded::from_str(query).unwrap_or_else(|_| BTreeMap::new()))
            .unwrap_or_default();

        let mut variables = Variables::default();

        // Method
        if self.method.clone().to_hyper() != request.method() {
            return None;
        }

        // Path
        let path = self.path.matches(request.uri().path())?;

        // Query
        let query = self.query_params.matches(query_params)?;

        // FIXME: Too much cloning is happening via merge_variables
        variables = merge_variables(variables, path);
        variables = merge_variables(variables, query);

        Some(PartialRequest {
            body: self.body.as_ref(),
            graphql_query: &self.graphql_query,
            variables,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct PartialRequest<'a> {
    body: Option<&'a String>,
    graphql_query: &'a String,
    variables: Variables,
}

impl<'a> PartialRequest<'a> {
    pub async fn into_request(self, request: Request) -> anyhow::Result<GraphQLRequest> {
        let mut variables = self.variables;
        if let Some(key) = self.body {
            let bytes = hyper::body::to_bytes(request.into_body()).await?;
            let body: ConstValue = serde_json::from_slice(&bytes)?;
            variables.insert(Name::new(key), body);
        }

        Ok(GraphQLRequest(
            async_graphql::Request::new(self.graphql_query).variables(variables),
        ))
    }
}

fn merge_variables(a: Variables, b: Variables) -> Variables {
    let mut variables = Variables::default();

    for (k, v) in a.iter() {
        variables.insert(k.clone(), v.clone());
    }

    for (k, v) in b.iter() {
        variables.insert(k.clone(), v.clone());
    }

    variables
}

#[cfg(test)]
mod tests {
    use async_graphql::parser::types::Directive;
    use maplit::btreemap;
    use pretty_assertions::assert_eq;

    use self::typed_variable::N;
    use super::*;

    const TEST_QUERY: &str = r#"
        query ($a: Int, $b: String, $c: Boolean, $d: Float, $v: String)
          @rest(method: "post", path: "/foo/$a", query: {b: $b, c: $c, d: $d}, body: $v) {
            value
          }
        "#;

    impl TypedVariable {
        fn string(name: &str) -> Self {
            Self::new(UrlParamType::String, name)
        }

        fn float(name: &str) -> Self {
            Self::new(UrlParamType::Number(N::Float), name)
        }

        fn boolean(name: &str) -> Self {
            Self::new(UrlParamType::Boolean, name)
        }
    }

    impl Path {
        fn new(segments: Vec<Segment>) -> Self {
            Self { segments }
        }
    }
    fn test_directive() -> Directive {
        async_graphql::parser::parse_query(TEST_QUERY)
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

    #[test]
    fn test_rest() {
        let directive = test_directive();
        let actual = Rest::try_from(&directive).unwrap();
        let expected = Rest::default()
            .path("/foo/$a".to_string())
            .method(Some(Method::POST))
            .query(
                btreemap! { "b".to_string() => "b".to_string(), "c".to_string() => "c".to_string(), "d".to_string() => "d".to_string() },
            )
            .body(Some("v".to_string()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_endpoint() {
        let endpoint = &Endpoint::try_new(TEST_QUERY).unwrap()[0];
        assert_eq!(endpoint.method, Method::POST);
        assert_eq!(
            endpoint.path,
            Path::new(vec![
                Segment::lit("foo"),
                Segment::param(UrlParamType::Number(N::Int), "a"),
            ])
        );
        assert_eq!(
            endpoint.query_params,
            QueryParams::from(vec![
                ("b", TypedVariable::string("b")),
                ("c", TypedVariable::boolean("c")),
                ("d", TypedVariable::float("d"))
            ])
        );
        assert_eq!(endpoint.body, Some("v".to_string()));
    }

    mod matches {
        use std::str::FromStr;

        use async_graphql::Variables;
        use async_graphql_value::{ConstValue, Name};
        use hyper::{Body, Method, Request, Uri, Version};
        use maplit::btreemap;
        use pretty_assertions::assert_eq;

        use crate::rest::endpoint::tests::TEST_QUERY;
        use crate::rest::endpoint::Endpoint;

        fn test_request(method: Method, uri: &str) -> anyhow::Result<hyper::Request<Body>> {
            Ok(Request::builder()
                .method(method)
                .uri(Uri::from_str(uri)?)
                .version(Version::HTTP_11)
                .body(Body::empty())?)
        }

        fn test_matches(query: &str, method: Method, uri: &str) -> Option<Variables> {
            let endpoint = &mut Endpoint::try_new(query).unwrap()[0];
            let request = test_request(method, uri).unwrap();

            endpoint.matches(&request).map(|req| req.variables)
        }

        #[test]
        fn test_valid() {
            let actual = test_matches(
                TEST_QUERY,
                Method::POST,
                "http://localhost:8080/foo/1?b=b&c=true&d=1.25",
            );
            let expected = &btreemap! {
                Name::new("a") => ConstValue::from(1),
                Name::new("b") => ConstValue::from("b"),
                Name::new("c") => ConstValue::from(true),
                Name::new("d") => ConstValue::from(1.25),
            };
            pretty_assertions::assert_eq!(actual.as_deref(), Some(expected))
        }

        #[test]
        fn test_path_not_match() {
            let actual = test_matches(
                TEST_QUERY,
                Method::POST,
                "http://localhost:8080/bar/1?b=b&c=true",
            );

            assert_eq!(actual, None);
            let actual = test_matches(
                TEST_QUERY,
                Method::POST,
                "http://localhost:8080/foo/1/nested?b=b&c=true",
            );

            assert_eq!(actual, None);
        }

        #[test]
        fn test_invalid_url_param() {
            let actual = test_matches(
                TEST_QUERY,
                Method::POST,
                "http://localhost:8080/foo/a?b=b&c=true",
            );
            pretty_assertions::assert_eq!(actual, None)
        }

        #[test]
        fn test_query_params_optional() {
            let actual = test_matches(TEST_QUERY, Method::POST, "http://localhost:8080/foo/1");
            let expected = &btreemap! {
                Name::new("a") => ConstValue::from(1),
            };
            pretty_assertions::assert_eq!(actual.as_deref(), Some(expected));

            let actual = test_matches(TEST_QUERY, Method::POST, "http://localhost:8080/foo/1/?b=b");
            let expected = &btreemap! {
                Name::new("a") => ConstValue::from(1),
                Name::new("b") => ConstValue::from("b"),
            };
            pretty_assertions::assert_eq!(actual.as_deref(), Some(expected))
        }

        #[test]
        fn test_invalid_query_param() {
            let actual = test_matches(
                TEST_QUERY,
                Method::POST,
                "http://localhost:8080/foo/1?b=b&c=c",
            );
            assert_eq!(actual, None)
        }

        #[test]
        fn test_method_not_match() {
            let actual = test_matches(
                TEST_QUERY,
                Method::GET,
                "http://localhost:8080/foo/1?b=b&c=true",
            );
            assert_eq!(actual, None)
        }
    }
}
