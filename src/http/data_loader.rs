use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_graphql::async_trait;
use async_graphql::dataloader::{DataLoader, HashMapCache, Loader, NoCache};
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;
use url::Url;

use crate::http::{HttpClient, Method, Response};
use crate::json::JsonLike;
#[derive(Debug)]
pub struct EndpointKey {
  pub request: reqwest::Request,
  pub match_key_value: ConstValue,
  pub match_path: Vec<String>,
  pub batching_enabled: bool,
  pub list: bool,
}
impl Hash for EndpointKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.request.url().hash(state);
    self.request.method().hash(state);
    // self.request.headers().hash(state);
  }
}

impl PartialEq for EndpointKey {
  fn eq(&self, other: &Self) -> bool {
    self.request.url() == other.request.url() && self.request.method() == other.request.method()
  }
}

impl Clone for EndpointKey {
  fn clone(&self) -> Self {
    EndpointKey {
      request: self.request.try_clone().unwrap(),
      match_key_value: self.match_key_value.clone(),
      match_path: self.match_path.clone(),
      batching_enabled: self.batching_enabled,
      list: self.list,
    }
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
    let unbatched_keys = keys
      .iter()
      .filter(|key| !key.batching_enabled)
      .map(|key| (*key).clone())
      .collect::<Vec<_>>();
    let futures: Vec<_> = unbatched_keys
      .iter()
      .map(|key| async {
        let result = self.client.clone().execute(key.clone().request).await;
        (key.clone(), result)
      })
      .collect();

    let results = join_all(futures).await;
    results.into_iter().map(|(key, result)| Ok((key, result?))).collect()
  }

  pub fn group_by_url_and_type(&self, keys: &[EndpointKey]) -> HashMap<Url, Vec<EndpointKey>> {
    keys
      .iter()
      .filter(|endpoint_key| endpoint_key.batching_enabled)
      .fold(HashMap::new(), |mut acc, key| {
        let group = acc.entry(key.clone().request.url().clone()).or_default();
        group.push(key.clone());
        acc
      })
  }

  async fn get_batched_results(
    &self,
    keys: &[EndpointKey],
  ) -> Vec<anyhow::Result<HashMap<EndpointKey, <HttpDataLoader as Loader<EndpointKey>>::Value>>> {
    let batched_key_groups = self.group_by_url_and_type(keys);
    join_all(
      batched_key_groups
        .iter()
        .map(|(url, keys)| self.get_batched_results_for_url(url.clone(), keys)),
    )
    .await
  }

  async fn get_batched_results_for_url(
    &self,
    url: Url,
    keys: &[EndpointKey],
  ) -> anyhow::Result<HashMap<EndpointKey, <HttpDataLoader as Loader<EndpointKey>>::Value>> {
    let req = reqwest::Request::new(Method::GET.into(), url);
    let response = self.client.clone().execute(req).await?;

    match &response.body {
      async_graphql::Value::List(list) => {
        #[allow(clippy::mutable_key_type)]
        let mut map: HashMap<EndpointKey, <HttpDataLoader as Loader<EndpointKey>>::Value> = HashMap::new();
        let mut body: ConstValue;
        for key in keys.iter() {
          let match_fn = |item: &&ConstValue| -> bool {
            if let Some(value) = item.get_path(&key.match_path) {
              value == &key.match_key_value
            } else {
              false
            }
          };

          if key.list {
            let items = list.iter().filter(match_fn).cloned().collect();
            body = async_graphql::Value::List(items);
          } else {
            body = list.iter().find(match_fn).cloned().unwrap_or(ConstValue::Null);
          }
          let response_for_key = Response { status: response.status, headers: response.headers.clone(), body };
          map.insert(key.clone(), response_for_key);
        }
        Ok(map)
      }
      _ => Ok(HashMap::new()), // TODO throw error
    }
  }
}

#[async_trait::async_trait]
impl Loader<EndpointKey> for HttpDataLoader {
  type Value = Response;
  type Error = Arc<anyhow::Error>;

  async fn load(&self, keys: &[EndpointKey]) -> async_graphql::Result<HashMap<EndpointKey, Self::Value>, Self::Error> {
    let batched_results = self.get_batched_results(keys).await;
    let unbatched_results = self.get_unbatched_results(keys).await;
    #[allow(clippy::mutable_key_type)]
    let mut all_results = HashMap::new();
    for result in batched_results {
      all_results.extend(result?);
    }
    all_results.extend(unbatched_results?);
    Ok(all_results)
  }
}
