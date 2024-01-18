use std::num::NonZeroU64;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use async_graphql_value::ConstValue;
use serde_json::{Number, Value};
use tailcall::json::JsonLike;
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
  async fn internal_insert(kv_store: KvStore, key: String, value: ConstValue, ttl: u64) -> Result<ConstValue> {
    kv_store
      .put(&key.to_string(), value.to_string())
      .map_err(to_anyhow)?
      .expiration_ttl(ttl)
      .execute()
      .await
      .map_err(to_anyhow)?;
    Ok(value)
  }
  async fn internal_get(kv_store: KvStore, key: String) -> Result<ConstValue> {
    let val = kv_store.get(&key).json::<Value>().await.map_err(to_anyhow)?;
    let val = val.ok_or(anyhow!("key not found"))?;
    Ok(ConstValue::from_json(val)?)
  }
}
// TODO: Needs fix
#[async_trait::async_trait]
impl Cache<u64, ConstValue> for CloudflareChronoCache {
  async fn insert<'a>(&'a self, key: u64, value: ConstValue, ttl: NonZeroU64) -> Result<ConstValue> {
    let kv_store = self.get_kv()?;
    async_std::task::spawn_local(Self::internal_insert(kv_store, key.to_string(), value, ttl.get())).await
  }

  async fn get<'a>(&'a self, key: &'a u64) -> Result<ConstValue> {
    let kv_store = self.get_kv()?;
    async_std::task::spawn_local(Self::internal_get(kv_store, key.to_string())).await
  }
}
