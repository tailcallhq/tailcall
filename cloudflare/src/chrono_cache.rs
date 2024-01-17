use std::hash::Hash;
use std::num::NonZeroU64;
use std::rc::Rc;

use anyhow::Result;
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

impl<K: Hash + Eq, V: Clone> ChronoCache<K, V> for CloudflareChronoCache {
  fn insert(&self, key: K, value: V, ttl: NonZeroU64) -> Result<V> {
    unimplemented!()
  }

  fn get(&self, key: &K) -> Result<V> {
    unimplemented!()
  }
}
