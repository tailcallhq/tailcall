use std::collections::HashMap;

use async_graphql::parser::types::{BaseType, Directive, ExecutableDocument, Type};
use async_graphql::{Name, Variables};
use async_graphql_value::{ConstValue, Value};
use derive_setters::Setters;

use serde::{Deserialize, Serialize};

use crate::directive::DirectiveCodec;
use crate::http::Method;
use crate::is_default;

#[derive(Clone, Debug, PartialEq)]
pub enum UrlParamType {
    String,
    Int,
    Boolean,
}

impl UrlParamType {
    fn to_value(&self, value: &str) -> anyhow::Result<ConstValue> {
        Ok(match self {
            Self::String => ConstValue::String(value.to_string()),

            // FIXME: this should decode to a numeric type instead of a string
            Self::Int => ConstValue::from(value),
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
                "Int" => Ok(Self::Int),
                "Boolean" => Ok(Self::Boolean),
                _ => Err(anyhow::anyhow!("unsupported type: {}", name)),
            },
            _ => Err(anyhow::anyhow!("unsupported type: {:?}", value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
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

    pub fn string(s: &str) -> Self {
        Self::Param(TypedVariable::string(s))
    }

    pub fn int(s: &str) -> Self {
        Self::Param(TypedVariable::int(s))
    }

    pub fn boolean(s: &str) -> Self {
        Self::Param(TypedVariable::boolean(s))
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Path {
    segments: Vec<Segment>,
}

#[derive(Debug)]
struct TypeMap(HashMap<String, Type>);
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
            if s.starts_with('$') {
                let key = &s[1..];
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

    fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }

    fn eval(&self, path: &str) -> anyhow::Result<Variables> {
        let mut variables = Variables::default();
        let mut path_segments = path.split('/').filter(|s| !s.is_empty());
        for segment in &self.segments {
            if let Some(path_segment) = path_segments.next() {
                if let Segment::Param(t_var) = segment {
                    let tpe = t_var.to_value(path_segment)?;
                    variables.insert(Name::new(t_var.name.clone()), tpe);
                }
            }
        }
        Ok(variables)
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Query {
    params: Vec<(String, TypedVariable)>,
}

impl From<Vec<(&str, TypedVariable)>> for Query {
    fn from(value: Vec<(&str, TypedVariable)>) -> Self {
        Self {
            params: value.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Setters)]
struct TypedVariable {
    type_of: UrlParamType,
    name: String,
    nullable: bool,
}

impl TypedVariable {
    fn new(tpe: UrlParamType, name: &str) -> Self {
        Self { type_of: tpe, name: name.to_string(), nullable: false }
    }

    fn string(name: &str) -> Self {
        Self::new(UrlParamType::String, name)
    }

    fn int(name: &str) -> Self {
        Self::new(UrlParamType::Int, name)
    }

    fn boolean(name: &str) -> Self {
        Self::new(UrlParamType::Boolean, name)
    }

    fn to_value(&self, value: &str) -> anyhow::Result<ConstValue> {
        self.type_of.to_value(value)
    }
}

impl Query {
    fn try_from_map(q: &TypeMap, map: HashMap<String, String>) -> anyhow::Result<Self> {
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

    fn eval(&self, query_params: HashMap<String, String>) -> anyhow::Result<Variables> {
        let mut variables = Variables::default();
        for (key, t_var) in &self.params {
            if let Some(query_param) = query_params.get(key) {
                let value = t_var.to_value(query_param)?;
                variables.insert(Name::new(t_var.name.clone()), value);
            }
        }
        Ok(variables)
    }
}

#[derive(Debug, Setters)]
pub struct Endpoint {
    method: Method,
    path: Path,
    query: Query,
    body: Option<String>,
    doc: ExecutableDocument,
    type_map: TypeMap,
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Setters)]
struct Rest {
    path: String,
    #[serde(default, skip_serializing_if = "is_default")]
    method: Option<Method>,
    #[serde(default, skip_serializing_if = "is_default")]
    query: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    body: Option<String>,
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
                    .collect::<HashMap<_, _>>(),
            );

            let rest = op.node.directives.iter().find_map(|d| {
                if d.node.name.node == Rest::directive_name() {
                    let rest = Rest::try_from(&d.node);
                    Some(rest)
                } else {
                    None
                }
            });

            if let Some(rest) = rest {
                let rest = rest?;
                let endpoint = Self {
                    method: rest.method.unwrap_or_default(),
                    path: Path::parse(&type_map, &rest.path)?,
                    query: Query::try_from_map(&type_map, rest.query)?,
                    body: rest.body,
                    doc: doc.clone(),
                    type_map,
                };
                endpoints.push(endpoint);
            }
        }

        Ok(endpoints)
    }

    pub fn eval(&self, request: &reqwest::Request) -> anyhow::Result<Variables> {
        let method = request.method();
        let path = request.url().path();
        let query_params = request
            .url()
            .query_pairs()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<HashMap<_, _>>();
        let body = request
            .body()
            .and_then(|b| b.as_bytes())
            .map(serde_json::from_slice::<ConstValue>);

        let mut variables = Variables::default();
        if self.method.clone().to_hyper() != method {
            return Ok(variables);
        }

        variables = merge_variables(variables, self.path.clone().eval(path)?);
        variables = merge_variables(variables, self.query.eval(query_params)?.clone());

        let body_param = self.body.clone();
        if let (Some(body), Some(key)) = (body, body_param) {
            variables.insert(Name::new(key), body?);
        }

        Ok(variables)
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
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use stripmargin::StripMargin;

    use super::*;

    fn test_query() -> String {
        r#"
        |query ($a: Int, $b: String, $c: Boolean, $d: String)
        |  @rest(method: "post", path: "/foo/$a", query: {b: $b, c: $c}, body: $d) {
        |    value
        |  }
        "#
        .strip_margin()
    }
    fn test_directive() -> Directive {
        async_graphql::parser::parse_query(test_query())
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
                hashmap! { "b".to_string() => "b".to_string(), "c".to_string() => "c".to_string() },
            )
            .body(Some("d".to_string()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_endpoint() {
        let endpoint = &Endpoint::try_new(test_query().as_str()).unwrap()[0];
        assert_eq!(endpoint.method, Method::POST);
        assert_eq!(
            endpoint.path,
            Path::new(vec![
                Segment::lit("foo"),
                Segment::param(UrlParamType::Int, "a")
            ])
        );
        assert_eq!(
            endpoint.query,
            Query::from(vec![
                ("b", TypedVariable::string("b")),
                ("c", TypedVariable::boolean("c")),
            ])
        );
        assert_eq!(endpoint.body, Some("d".to_string()));
    }
}
