use std::collections::HashMap;
use std::ops::DerefMut;

use async_graphql::parser::types::ExecutableDocument;
use async_graphql_value::ConstValue;
use tailcall_valid::Validator;

use super::{transform, Builder, OperationPlan, Result, Variables};
use crate::core::transform::TransformerOps;
use crate::core::Transform;
use crate::core::{async_graphql_hyper::GraphQLRequest, blueprint::Blueprint};

#[derive(Debug, Clone)]
pub struct Request<V> {
    pub query: String,
    pub operation_name: Option<String>,
    pub variables: Variables<V>,
    pub extensions: HashMap<String, V>,
    pub parsed_query: ExecutableDocument,
}

// NOTE: This is hot code and should allocate minimal memory
impl TryFrom<async_graphql::Request> for Request<ConstValue> {
    type Error = super::Error;

    fn try_from(mut value: async_graphql::Request) -> Result<Self> {
        let variables = std::mem::take(value.variables.deref_mut());

        Ok(Self {
            parsed_query: value.parsed_query()?.clone(),
            query: value.query,
            operation_name: value.operation_name,
            variables: Variables::from_iter(variables.into_iter().map(|(k, v)| (k.to_string(), v))),
            extensions: value.extensions.0,
        })
    }
}

impl TryFrom<GraphQLRequest> for Request<ConstValue> {
    type Error = super::Error;

    fn try_from(value: GraphQLRequest) -> Result<Self> {
        Self::try_from(value.0)
    }
}

impl Request<ConstValue> {
    pub fn create_plan(
        &self,
        blueprint: &Blueprint,
    ) -> Result<OperationPlan<async_graphql_value::Value>> {
        let builder = Builder::new(blueprint, &self.parsed_query);
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
            parsed_query: async_graphql::parser::parse_query(query).unwrap(),
        }
    }

    pub fn variables(self, vars: impl IntoIterator<Item = (String, V)>) -> Self {
        Self { variables: Variables::from_iter(vars), ..self }
    }
}
