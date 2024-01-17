use std::hash::Hash;
use std::num::NonZeroU64;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use async_graphql_value::ConstValue;
use hyper::body::HttpBody;
use serde_json::{Number, Value};
use tailcall::json::JsonLike;
use tailcall::ChronoCache;

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
  fn get_kv(&self) -> Result<worker::kv::KvStore> {
    self.env.kv("TMP_KV").map_err(to_anyhow)
  }
}
// TODO: Needs fix
#[async_trait::async_trait]
impl ChronoCache<u64, ConstValue> for CloudflareChronoCache {
  async fn insert<'a>(&'a self, key: u64, value: ConstValue, ttl: NonZeroU64) -> Result<ConstValue> {
    let mut json = serde_json::Map::new();
    let ttl = ttl.get();
    json.insert("ttl".to_string(), Value::Number(Number::from(ttl)));
    let value_str = value.as_str_ok().map_err(to_anyhow)?;
    json.insert("value".to_string(), serde_json::from_str(value_str)?);
    let kv_store = self.get_kv()?;
    let put_options = kv_store.put(&key.to_string(), Value::Object(json)).map_err(to_anyhow)?;
    async_std::task::spawn_local(put_options.execute())
      .await
      .map_err(to_anyhow)?;
    Ok(value)
  }

  async fn get<'a>(&'a self, key: &'a u64) -> Result<ConstValue> {
    let kv_store = self.get_kv()?;
    let val = kv_store
      .get(&key.to_string())
      .json::<Value>()
      .await
      .map_err(to_anyhow)?;
    let val = val.ok_or(anyhow!("key not found"))?;
    Ok(ConstValue::from_json(val)?)
  }
}
