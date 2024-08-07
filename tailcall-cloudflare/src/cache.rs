use std::num::NonZeroU64;
use std::rc::Rc;

use async_graphql_value::ConstValue;
use serde_json::Value;
use tailcall::core::ir::model::IoId;
use tailcall::core::{cache, Cache};
use worker::kv::KvStore;

pub struct CloudflareChronoCache {
    env: Rc<worker::Env>,
}

unsafe impl Send for CloudflareChronoCache {}

unsafe impl Sync for CloudflareChronoCache {}

impl CloudflareChronoCache {
    pub fn init(env: Rc<worker::Env>) -> Self {
        Self { env }
    }
    fn get_kv(&self) -> Result<KvStore, cache::Error> {
        self.env
            .kv("TMP_KV")
            .map_err(|e| cache::Error::Kv(e.to_string()))
    }
}
// TODO: Needs fix
#[async_trait::async_trait]
impl Cache for CloudflareChronoCache {
    type Key = IoId;
    type Value = ConstValue;
    async fn set<'a>(
        &'a self,
        key: IoId,
        value: ConstValue,
        ttl: NonZeroU64,
    ) -> Result<(), cache::Error> {
        let kv_store = self.get_kv()?;
        let ttl = ttl.get();
        async_std::task::spawn_local(async move {
            kv_store
                .put(&key.as_u64().to_string(), value.to_string())
                .map_err(|e| cache::Error::Kv(e.to_string()))?
                .expiration_ttl(ttl)
                .execute()
                .await
                .map_err(|e| cache::Error::Kv(e.to_string()))
        })
        .await
    }

    async fn get<'a>(&'a self, key: &'a IoId) -> Result<Option<Self::Value>, cache::Error> {
        let kv_store = self.get_kv()?;
        let key = key.as_u64().to_string();
        async_std::task::spawn_local(async move {
            let val = kv_store
                .get(&key)
                .json::<Value>()
                .await
                .map_err(|e| cache::Error::Kv(e.to_string()))?;
            Ok(val.map(ConstValue::from_json).transpose()?)
        })
        .await
    }

    fn hit_rate(&self) -> Option<f64> {
        None
    }
}
