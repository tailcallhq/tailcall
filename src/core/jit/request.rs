use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use derive_setters::Setters;
use serde::Deserialize;

use super::{Builder, Error, ExecutionPlan, Result};
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

impl Hash for Request<async_graphql::Value> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.query.hash(state);
        self.operation_name.hash(state);
        for (name, value) in self.variables.iter() {
            name.hash(state);
            value.to_string().hash(state);
        }
    }
}

// we already have a struct named Data in store
// anyways I don't think we need this struct
#[derive(Default, Debug)]
pub struct Extras(pub HashMap<TypeId, Box<dyn Any + Sync + Send>>);

impl Extras {
    pub fn insert<T: Any + Sync + Send>(&mut self, value: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: Any + Sync + Send>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
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
            data: Default::default(),
        }
    }
}
