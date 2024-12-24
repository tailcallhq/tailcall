use std::collections::HashMap;
use std::ops::DerefMut;

use async_graphql_value::ConstValue;
use serde::Deserialize;
use tailcall_valid::Validator;

use super::{transform, Builder, OperationPlan, Result, Variables};
use crate::core::blueprint::Blueprint;
use crate::core::transform::TransformerOps;
use crate::core::Transform;

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

// NOTE: This is hot code and should allocate minimal memory
impl From<async_graphql::Request> for Request<ConstValue> {
    fn from(mut value: async_graphql::Request) -> Self {
        let variables = std::mem::take(value.variables.deref_mut());

        Self {
            query: value.query,
            operation_name: value.operation_name,
            variables: Variables::from_iter(variables.into_iter().map(|(k, v)| (k.to_string(), v))),
            extensions: value.extensions.0,
        }
    }
}

impl Request<ConstValue> {
    pub fn create_plan(
        &self,
        blueprint: &Blueprint,
    ) -> Result<OperationPlan<async_graphql_value::Value>> {
        let doc = async_graphql::parser::parse_query(&self.query)?;
        let builder = Builder::new(blueprint, &doc);
        let plan = builder.build(self.operation_name.as_deref())?;

        transform::CheckConst::new()
            .pipe(transform::CheckProtected::new())
            .pipe(transform::AuthPlanner::new())
            .pipe(transform::CheckDedupe::new())
            .pipe(transform::CheckCache::new())
            .pipe(transform::GraphQL::new())
            .transform(plan)
            .to_result()
            // both transformers are infallible right now
            // but we can't just unwrap this in stable rust
            // so convert to the Unknown error
            .map_err(|_| super::Error::Unknown)
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
