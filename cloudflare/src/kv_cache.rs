use std::num::NonZeroU64;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use async_graphql_value::ConstValue;
use serde_json::Value;
use tailcall::Cache;
use worker::kv::KvStore;

use crate::to_anyhow;

pub struct CloudflareChronoCache {
  env: Rc<worker::Env>,
}

unsafe impl Send for CloudflareChronoCache {}

unsafe impl Sync for CloudflareChronoCache {}

impl CloudflareChronoCache {
  pub fn init(env: Rc<worker::Env>) -> Self {
    Self { env }
  }
  fn get_kv(&self) -> Result<KvStore> {
    self.env.kv("TMP_KV").map_err(to_anyhow)
  }
}
// TODO: Needs fix
#[async_trait::async_trait]
impl Cache<u64, ConstValue> for CloudflareChronoCache {
  async fn insert<'a>(&'a self, key: u64, value: ConstValue, ttl: NonZeroU64) -> Result<ConstValue> {
    let kv_store = self.get_kv()?;
    let ttl = ttl.get();
    async_std::task::spawn_local(async move {
      kv_store
        .put(&key.to_string(), value.to_string())
        .map_err(to_anyhow)?
        .expiration_ttl(ttl)
        .execute()
        .await
        .map_err(to_anyhow)?;
      anyhow::Ok(value)
    })
    .await
  }

  async fn get<'a>(&'a self, key: &'a u64) -> Result<ConstValue> {
    let kv_store = self.get_kv()?;
    let key = key.to_string();
    async_std::task::spawn_local(async move {
      let val = kv_store.get(&key).json::<Value>().await.map_err(to_anyhow)?;
      let val = val.ok_or(anyhow!("key not found"))?;
      Ok(ConstValue::from_json(val)?)
    })
    .await
  }
}
