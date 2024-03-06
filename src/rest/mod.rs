use std::collections::{BTreeMap, HashMap};

use async_graphql::{
    parser::types::{BaseType, ExecutableDocument, Type},
    Name, Value, Variables,
};
use derive_setters::Setters;
use hyper::body::Bytes;

use crate::http::Method;

#[derive(Clone, Debug, PartialEq)]
pub enum VariableType {
    String,
    Int,
    Boolean,
}

impl VariableType {
    fn to_value(&self, value: &str) -> anyhow::Result<Value> {
        Ok(match self {
            Self::String => Value::String(value.to_string()),

            // FIXME: this should decode to a numeric type instead of a string
            Self::Int => Value::from(value),
            Self::Boolean => Value::Boolean(value.parse()?),
        })
    }
}

impl TryFrom<&Type> for VariableType {
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
    pub fn literal(s: &str) -> Self {
        Self::Literal(s.to_string())
    }

    pub fn param(t: VariableType, s: &str) -> Self {
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
                let t = VariableType::try_from(value)?;
                segments.push(Segment::param(t, &key));
            } else {
                segments.push(Segment::literal(s));
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
    type_of: VariableType,
    name: String,
    nullable: bool,
}

impl TypedVariable {
    fn new(tpe: VariableType, name: &str) -> Self {
        Self { type_of: tpe, name: name.to_string(), nullable: false }
    }

    fn string(name: &str) -> Self {
        Self::new(VariableType::String, name)
    }

    fn int(name: &str) -> Self {
        Self::new(VariableType::Int, name)
    }

    fn boolean(name: &str) -> Self {
        Self::new(VariableType::Boolean, name)
    }

    fn to_value(&self, value: &str) -> anyhow::Result<Value> {
        self.type_of.to_value(value)
    }
}

impl Query {
    fn try_from_map(q: &TypeMap, map: HashMap<String, String>) -> anyhow::Result<Self> {
        let mut params = Vec::new();
        for (k, v) in map {
            if k.starts_with('$') {
                let key = &k[1..];
                let t = VariableType::try_from(
                    q.get(&key)
                        .ok_or(anyhow::anyhow!("undefined query param: {}", key))?,
                )?;
                params.push((k, TypedVariable::new(t, &v)));
            } else {
                return Err(anyhow::anyhow!(
                    "query param: {} should map to a $variable",
                    k
                ));
            }
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

impl Endpoint {
    pub fn new(doc: ExecutableDocument) -> Self {
        let type_map = TypeMap(
            doc.operations
                .iter()
                .flat_map(|(_, op)| {
                    op.node.variable_definitions.iter().map(|pos| {
                        (
                            pos.node.name.node.to_string(),
                            pos.node.var_type.node.clone(),
                        )
                    })
                })
                .collect::<HashMap<_, _>>(),
        );

        Self {
            method: Default::default(),
            path: Default::default(),
            query: Default::default(),
            body: Default::default(),
            doc,
            type_map,
        }
    }

    pub fn with_query_params(
        mut self,
        query_params: HashMap<String, String>,
    ) -> anyhow::Result<Self> {
        self.query = Query::try_from_map(&self.type_map, query_params)?;
        Ok(self)
    }

    pub fn with_path_str(mut self, path: &str) -> anyhow::Result<Self> {
        self.path = Path::parse(&self.type_map, path)?;
        Ok(self)
    }

    pub fn eval(
        &self,
        method: Method,
        path: &str,
        query_params: HashMap<String, String>,
        body: Option<Bytes>,
    ) -> anyhow::Result<Variables> {
        let mut variables = Variables::default();
        if self.method != method {
            return Ok(variables);
        }

        let body_param = self.body.clone();
        variables = merge_variables(variables, self.path.clone().eval(path)?);
        variables = merge_variables(variables, self.query.eval(query_params)?.clone());
        if let (Some(body), Some(key)) = (body, body_param) {
            let value = serde_json::from_slice::<Value>(&body)?;
            variables.insert(Name::new(key), value);
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
    use super::*;
    use pretty_assertions::assert_eq;

    fn new_type(name: &str) -> Type {
        Type { base: BaseType::Named(Name::new(name)), nullable: false }
    }

    #[test]
    fn test_parse_path() {
        let inputs = vec![
            ("/users", vec![Segment::literal("users")]),
            (
                "/users/$id",
                vec![Segment::literal("users"), Segment::int("id")],
            ),
            (
                "/users/$id/posts",
                vec![
                    Segment::literal("users"),
                    Segment::int("id"),
                    Segment::literal("posts"),
                ],
            ),
        ];

        let t_map = TypeMap::from(vec![("id", new_type("Int")), ("name", new_type("String"))]);
        for (input, expected) in inputs {
            let path = Path::parse(&t_map, input).unwrap();
            assert_eq!(path, Path::new(expected));
        }
    }

    #[test]
    fn test_from_query() {
        let type_map = TypeMap::from(vec![("id", new_type("Int")), ("name", new_type("String"))]);
        let inputs = vec![
            (vec![], Query { params: vec![] }),
            (
                vec![("id", "$name")],
                Query::from(vec![("id", TypedVariable::int("name"))]),
            ),
            (
                vec![("id", "$id")],
                Query::from(vec![("id", TypedVariable::int("id"))]),
            ),
        ];

        for (input, expected) in inputs {
            let query = Query::try_from_map(
                &type_map,
                input
                    .into_iter()
                    .map(|(a, b)| (a.to_owned(), b.to_owned()))
                    .collect(),
            )
            .unwrap();
            assert_eq!(query, expected);
        }
    }
}
