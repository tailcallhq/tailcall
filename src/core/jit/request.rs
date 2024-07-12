use std::collections::HashMap;
use std::ops::DerefMut;

use async_graphql_value::ConstValue;
use serde::Deserialize;

use super::input_resolver::InputResolver;
use super::{Builder, ExecutionPlan, Result, Variables};
use crate::core::blueprint::Blueprint;

#[derive(Debug, Deserialize, Clone)]
pub struct Request<V> {
    #[serde(default)]
    pub query: String,
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub variables: Variables<V>,
    #[serde(default)]
    pub extensions: HashMap<String, V>,
}

impl From<async_graphql::Request> for Request<ConstValue> {
    fn from(mut value: async_graphql::Request) -> Self {
        let variables = std::mem::take(value.variables.deref_mut());

        Self {
            query: value.query,
            operation_name: value.operation_name,
            variables: Variables::from_iter(variables.into_iter().map(|(k, v)| (k.to_string(), v))),
            extensions: value.extensions,
        }
    }
}

impl Request<ConstValue> {
    pub fn try_new(&self, blueprint: &Blueprint) -> Result<ExecutionPlan<ConstValue>> {
        let doc = async_graphql::parser::parse_query(&self.query)?;
        let builder = Builder::new(blueprint, doc);
        let plan = builder.build()?;
        let input_resolver = InputResolver::new(plan);

        // TODO: operation from [ExecutableDocument] could contain definitions for
        // default values of arguments. That info should be passed to
        // [InputResolver] to resolve defaults properly
        Ok(input_resolver.resolve_input(&self.variables)?)
    }
}

impl<V> Request<V> {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            operation_name: None,
            variables: Variables::new(),
            extensions: HashMap::new(),
        }
    }

    pub fn variables(self, vars: impl IntoIterator<Item = (String, V)>) -> Self {
        Self { variables: Variables::from_iter(vars), ..self }
    }
}
