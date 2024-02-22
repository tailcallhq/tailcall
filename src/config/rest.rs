use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter, Write};
use std::hash::Hasher;
use std::sync::Arc;

use async_graphql::parser::types::{BaseType, Type as GQLType};
use async_graphql_value::ConstValue;
use itertools::{EitherOrBoth, Itertools};
use serde::de::Visitor;
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::de::StrRead;

use crate::async_graphql_hyper::GraphQLRequest;
use crate::blueprint::OperationQuery;
use crate::http::Method;

#[derive(Clone, Debug, Default)]
pub struct RestApis(pub BTreeMap<Rest, String>);

impl RestApis {
    pub fn create_operations(&self) -> Vec<OperationQuery> {
        self.0
            .iter()
            .map(|(k, v)| {
                let variables = ConstValue::Object(
                    k.path
                        .variables()
                        .map(|var| {
                            (
                                async_graphql::Name::new(var.name.as_ref()),
                                var.default_value(),
                            )
                        })
                        .collect(),
                );
                let variables = async_graphql::Variables::from_value(variables);
                OperationQuery::new_with_variables(v.into(), "".into(), variables)
            })
            .collect()
    }

    pub fn dispatch_path(
        &self,
        method: hyper::Method,
        path: &str,
    ) -> anyhow::Result<GraphQLRequest> {
        let path = format!("\"{path}\"");
        let mut deserializer = serde_json::Deserializer::new(StrRead::new(path.as_str()));
        let path = RestPath::deserialize(&mut deserializer)?;
        let req_rest = Rest { method: method.try_into()?, path };

        let (rest, query) = self
            .0
            .get_key_value(&req_rest)
            .map(|(k, v)| (k.clone(), v))
            .ok_or(anyhow::anyhow!("path not found"))?;
        let vars_json = rest.path.extract_variable_values_from(req_rest.path);

        let req = async_graphql::Request::new(query)
            .variables(async_graphql::Variables::from_json(vars_json));

        Ok(GraphQLRequest(req))
    }
}

impl PartialEq for RestApis {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .all(|(rest, query)| other.0.get(rest).map_or(false, |q| q.eq(query)))
    }
}

impl Eq for RestApis {}

impl RestApis {
    pub fn merge_right(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }

    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(&mut self, rest: Rest, query: impl Into<String>) {
        self.0.insert(rest, query.into());
    }
}

#[derive(
    Clone, Debug, PartialEq, Deserialize, PartialOrd, Ord, Serialize, Eq, schemars::JsonSchema,
)]
/// The @rest operator creates a rest api for the operation it is applied to
#[serde(rename_all = "camelCase")]
pub struct Rest {
    /// Specifies the path for the rest api, relative to the base url.
    pub path: RestPath,
    /// Specifies the HTTP Method for the rest api
    #[serde(default)]
    pub method: Method,
}

#[derive(Clone, Debug, schemars::JsonSchema)]
pub struct RestPath {
    tokens: Vec<Token>,
}

impl PartialEq for RestPath {
    fn eq(&self, other: &Self) -> bool {
        self.tokens
            .iter()
            .zip_longest(other.tokens.iter())
            .all(|either_or_both| match either_or_both {
                EitherOrBoth::Both(Token::Static(a), Token::Static(b)) => a.eq(b),
                EitherOrBoth::Both(Token::Variable { .. }, _) => true,
                EitherOrBoth::Both(_, Token::Variable { .. }) => true,
                _ => false,
            })
    }
}

impl Eq for RestPath {}

impl PartialOrd for RestPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RestPath {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tokens.len().cmp(&other.tokens.len()) {
            Ordering::Equal => self
                .tokens
                .iter()
                .zip(other.tokens.iter())
                .find_map(|(a, b)| match (a, b) {
                    (Token::Static(a), Token::Static(b)) => Some(a.cmp(b)),
                    _ => None,
                })
                .unwrap_or(Ordering::Equal),
            res => res,
        }
    }
}

impl RestPath {
    pub fn variables_mut(&mut self) -> impl Iterator<Item = &mut Variable> {
        self.tokens.iter_mut().filter_map(|token| match token {
            Token::Variable(var) => Some(var),
            _ => None,
        })
    }

    pub fn variables(&self) -> impl Iterator<Item = &Variable> {
        self.tokens.iter().filter_map(|token| match token {
            Token::Variable(var) => Some(var),
            _ => None,
        })
    }

    pub fn extract_variable_values_from(self, req_path: RestPath) -> serde_json::Value {
        let mut values = serde_json::Map::new();
        for (token, req_token) in self.tokens.into_iter().zip(req_path.tokens.into_iter()) {
            if let (Token::Variable(var), Token::Static(val)) = (token, req_token) {
                values.insert(
                    var.name.to_string(),
                    serde_json::Value::String(val.to_string()),
                );
            }
        }
        serde_json::Value::Object(values)
    }
}

impl<'de> Deserialize<'de> for RestPath {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TokensVisitor;

        impl<'de> Visitor<'de> for TokensVisitor {
            type Value = RestPath;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a valid path")
            }

            fn visit_str<E>(self, mut v: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                if !v.is_empty() && &v[v.len() - 1..] == "/" {
                    v = &v[..v.len() - 1];
                }

                let mut tokens = v.split('/');

                tokens
                    .next()
                    .filter(|token| token.is_empty())
                    .ok_or(E::custom("path should start with \"/\""))?;

                let tokens = tokens
                    .map(|val| {
                        if let Some(var_name) = val.strip_prefix('$') {
                            let var = Variable { name: var_name.into(), typ: None };
                            Token::Variable(var)
                        } else {
                            Token::Static(val.into())
                        }
                    })
                    .collect();
                Ok(RestPath { tokens })
            }
        }

        deserializer.deserialize_str(TokensVisitor)
    }
}

impl Serialize for RestPath {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut result_str = String::new();
        self.tokens
            .iter()
            .map(|token| write!(&mut result_str, "{token}"))
            .collect::<anyhow::Result<Vec<_>, _>>()
            .map_err(|err| <S::Error as ser::Error>::custom(format!("{err}")))?;

        serializer.serialize_str(&result_str)
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Eq, schemars::JsonSchema)]
pub enum Token {
    Static(Arc<str>),
    Variable(Variable),
}

#[derive(Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct Variable {
    pub name: Arc<str>,
    #[serde(skip)]
    pub typ: Option<GQLType>,
}

pub fn gql_type_default_value(typ: &GQLType) -> ConstValue {
    match typ {
        GQLType { nullable: true, .. } => ConstValue::Null,
        GQLType { base, .. } => match base {
            BaseType::Named(name) => match name.as_str() {
                "Int" => 0.into(),
                "String" => "".into(),
                "Boolean" => false.into(),
                "Float" => 0.0.into(),
                _ => ConstValue::Null,
            },
            BaseType::List(typ) => ConstValue::List(vec![gql_type_default_value(typ.as_ref())]),
        },
    }
}

impl Variable {
    pub fn default_value(&self) -> ConstValue {
        match &self.typ {
            None => ConstValue::Null,
            Some(typ) => gql_type_default_value(typ),
        }
    }
}

impl std::hash::Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Token::Static(val) => val.to_string(),
            Token::Variable(var) => format!("${}", var.name),
        };
        write!(f, "{}", str)
    }
}
