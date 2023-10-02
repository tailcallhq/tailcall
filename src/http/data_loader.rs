use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_graphql::async_trait;
use async_graphql::dataloader::{DataLoader, HashMapCache, Loader, NoCache};
use async_graphql::futures_util::future::join_all;

use crate::http::{HttpClient, Response};

#[derive(Debug)]
pub struct EndpointKey(reqwest::Request, Vec<String>);

impl EndpointKey {
  pub fn new(req: reqwest::Request, headers: Vec<String>) -> Self {
    EndpointKey(req, headers)
  }
}
impl Deref for EndpointKey {
  type Target = reqwest::Request;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl Hash for EndpointKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.url().hash(state);
    self.method().hash(state);
    for (name, value) in self.headers().iter() {
      name.hash(state);
      value.hash(state);
    }
  }
}

impl PartialEq for EndpointKey {
  fn eq(&self, other: &Self) -> bool {
    let mut hasher_self = DefaultHasher::new();
    self.hash(&mut hasher_self);
    let hash_self = hasher_self.finish();

    let mut hasher_other = DefaultHasher::new();
    other.hash(&mut hasher_other);
    let hash_other = hasher_other.finish();

    hash_self == hash_other
  }
}

impl Clone for EndpointKey {
  fn clone(&self) -> Self {
    let mut req = reqwest::Request::new(self.method().clone(), self.url().clone());
    req.headers_mut().extend(self.headers().clone());
    EndpointKey(req, self.1.clone())
  }
}

impl Eq for EndpointKey {}
#[derive(Default, Clone)]
pub struct HttpDataLoader {
  pub client: HttpClient,
}

impl HttpDataLoader {
  pub fn new(client: HttpClient) -> Self {
    HttpDataLoader { client }
  }

  pub fn to_async_data_loader(self) -> DataLoader<HttpDataLoader, HashMapCache> {
    DataLoader::with_cache(self, tokio::spawn, HashMapCache::new()).delay(Duration::from_millis(0))
  }

  pub fn to_async_data_loader_options(self, delay: usize, max_size: usize) -> DataLoader<HttpDataLoader, NoCache> {
    DataLoader::new(self, tokio::spawn)
      .delay(Duration::from_millis(delay as u64))
      .max_batch_size(max_size)
  }

  pub async fn get_unbatched_results(
    &self,
    keys: &[EndpointKey],
  ) -> Result<HashMap<EndpointKey, <HttpDataLoader as Loader<EndpointKey>>::Value>> {
    let futures: Vec<_> = keys
      .iter()
      .map(|key| async {
        let result = self.client.clone().execute(key.clone().0).await;
        (key.clone(), result)
      })
      .collect();

    let results = join_all(futures).await;
    results.into_iter().map(|(key, result)| Ok((key, result?))).collect()
  }
}

#[async_trait::async_trait]
impl Loader<EndpointKey> for HttpDataLoader {
  type Value = Response;
  type Error = Arc<anyhow::Error>;

  async fn load(&self, keys: &[EndpointKey]) -> async_graphql::Result<HashMap<EndpointKey, Self::Value>, Self::Error> {
    #[allow(clippy::mutable_key_type)]
    let results = self.get_unbatched_results(keys).await?;
    Ok(results)
  }
}
