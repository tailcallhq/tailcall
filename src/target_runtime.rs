use std::sync::Arc;

use async_graphql_value::ConstValue;

use crate::{Cache, EnvIO, FileIO, HttpIO};

#[derive(Clone)]
pub struct TargetRuntime {
    pub http: Arc<dyn HttpIO>,
    pub http2_only: Arc<dyn HttpIO>,
    pub env: Arc<dyn EnvIO>,
    pub file: Arc<dyn FileIO>,
    pub cache: Arc<dyn Cache<Key = u64, Value = ConstValue>>,
}
