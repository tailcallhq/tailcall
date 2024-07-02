use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use derive_setters::Setters;
use serde::Deserialize;

use super::{Builder, ConstValueExecutor, Error, ExecutionPlan, Response, Result};
use crate::core::app_context::AppContext;
use crate::core::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};
use crate::core::blueprint::Blueprint;

#[derive(Debug, Deserialize, Setters)]
pub struct Request<Value> {
    #[serde(default)]
    pub query: String,
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub variables: HashMap<String, Value>,
    #[serde(default)]
    pub extensions: HashMap<String, Value>,

    #[serde(skip)]
    pub data: Extras,
}

impl<Value> Hash for Request<Value> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.query.hash(state);
        self.operation_name.hash(state);
        for (name, _value) in self.variables.iter() {
            name.hash(state);
            // value.to_string().hash(state);
        }
    }
}

// we already have a struct named Data in store
#[derive(Default, Debug)]
pub struct Extras(pub HashMap<TypeId, Box<dyn Any + Sync + Send>>);

impl From<Result<Response<async_graphql::Value, Error>>> for GraphQLResponse {
    fn from(val: Result<Response<async_graphql::Value, Error>>) -> Self {
        todo!()
    }
}

#[async_trait::async_trait]
impl GraphQLRequestLike for Request<async_graphql::Value> {
    type Response = Result<Response<async_graphql::Value, Error>>;

    fn data<D: Any + Clone + Send + Sync>(mut self, data: D) -> Self {
        self.data.0.insert(TypeId::of::<D>(), Box::new(data));
        self
    }

    async fn execute<E>(self, _: &E, app_ctx: Option<Arc<AppContext>>) -> Self::Response
    where
        E: async_graphql::Executor,
    {
        if app_ctx.is_none() {
            return Err(Error::BuildError("AppContext is missing".to_string()));
        }
        let exec = ConstValueExecutor::new(&self, app_ctx.unwrap())?;
        let resp = exec.execute(self).await;
        Ok(resp)
    }

    // we do not need to implement this method
    fn parse_query(&mut self) -> Option<&ExecutableDocument> {
        todo!()
    }
}

impl<Value> Request<Value> {
    pub fn try_new(&self, blueprint: &Blueprint) -> Result<ExecutionPlan> {
        let doc = async_graphql::parser::parse_query(&self.query)?;
        let builder = Builder::new(blueprint, doc);
        builder.build().map_err(Error::BuildError)
    }
}

impl<A> Request<A> {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            operation_name: None,
            variables: HashMap::new(),
            extensions: HashMap::new(),
            data: Extras(HashMap::new()),
        }
    }
}
