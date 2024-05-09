use std::num::NonZeroU64;

use async_graphql_value::ConstValue;
use tailcall::Cache;

pub struct WasmCache {}

impl WasmCache {
    pub fn init() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Cache for WasmCache {
    type Key = u64;
    type Value = ConstValue;

    async fn set<'a>(&'a self, _: Self::Key, _: Self::Value, _: NonZeroU64) -> anyhow::Result<()> {
        todo!()
    }

    async fn get<'a>(&'a self, _: &'a Self::Key) -> anyhow::Result<Option<Self::Value>> {
        todo!()
    }

    fn hit_rate(&self) -> Option<f64> {
        todo!()
    }
}
