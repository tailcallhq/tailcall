use std::sync::Arc;

use async_graphql_value::ConstValue;

use crate::{Cache, EnvIO, FileIO, HttpIO};

/// The TargetRuntime struct unifies the available runtime-specific
/// IO implementations. This is used to reduce piping IO structs all
/// over the codebase.
#[derive(Clone)]
pub struct TargetRuntime {
    pub http: Arc<dyn HttpIO>,
    pub http2_only: Arc<dyn HttpIO>,
    pub env: Arc<dyn EnvIO>,
    pub file: Arc<dyn FileIO>,
    pub cache: Arc<dyn Cache<Key = u64, Value = ConstValue>>,
}
