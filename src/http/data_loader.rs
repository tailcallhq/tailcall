use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_graphql::async_trait;
use async_graphql::dataloader::{DataLoader, HashMapCache, Loader, NoCache};
use async_graphql::futures_util::future::join_all;

use crate::http::{EndpointKey, HttpClientTrait, Response};

#[derive(Default, Clone)]
pub struct HttpDataLoader<C>
where
  C: HttpClientTrait + Send + Sync + 'static + Clone,
{
  pub client: C,
}
impl<C: HttpClientTrait + Send + Sync + 'static + Clone> HttpDataLoader<C> {
  pub fn new(client: C) -> Self {
    HttpDataLoader { client }
  }

  pub fn to_async_data_loader(self) -> DataLoader<HttpDataLoader<C>, HashMapCache> {
    DataLoader::with_cache(self, tokio::spawn, HashMapCache::new()).delay(Duration::from_millis(0))
  }

  pub fn to_async_data_loader_options(self, delay: usize, max_size: usize) -> DataLoader<HttpDataLoader<C>, NoCache> {
    DataLoader::new(self, tokio::spawn)
      .delay(Duration::from_millis(delay as u64))
      .max_batch_size(max_size)
  }

  pub async fn get_unbatched_results(
    &self,
    keys: &[EndpointKey],
  ) -> Result<HashMap<EndpointKey, <HttpDataLoader<C> as Loader<EndpointKey>>::Value>> {
    let futures: Vec<_> = keys
      .iter()
      .map(|key| async {
        let result = self.client.execute(key.request()).await;
        (key.clone(), result)
      })
      .collect();

    let results = join_all(futures).await;
    results.into_iter().map(|(key, result)| Ok((key, result?))).collect()
  }
}

#[async_trait::async_trait]
impl<C: HttpClientTrait + Send + Sync + 'static + Clone> Loader<EndpointKey> for HttpDataLoader<C> {
  type Value = Response;
  type Error = Arc<anyhow::Error>;

  async fn load(&self, keys: &[EndpointKey]) -> async_graphql::Result<HashMap<EndpointKey, Self::Value>, Self::Error> {
    #[allow(clippy::mutable_key_type)]
    let results = self.get_unbatched_results(keys).await?;
    Ok(results)
  }
}

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicUsize, Ordering};

  use super::*;
  use crate::http::EndpointKey;

  #[derive(Clone)]
  struct MockHttpClient {
    // To keep track of number of times execute is called
    request_count: Arc<AtomicUsize>,
  }

  #[async_trait::async_trait]
  impl HttpClientTrait for MockHttpClient {
    async fn execute(&self, _req: reqwest::Request) -> Result<Response> {
      self.request_count.fetch_add(1, Ordering::SeqCst);
      // You can mock the actual response as per your need
      Ok(Response::default())
    }
  }
  #[tokio::test]
  async fn test_load_function() {
    let client = MockHttpClient { request_count: Arc::new(AtomicUsize::new(0)) };

    let loader = HttpDataLoader { client: client.clone() };
    let loader = loader.to_async_data_loader_options(1, 1000);

    let request = reqwest::Request::new(reqwest::Method::GET, "http://example.com".parse().unwrap());
    let headers_to_consider = vec!["Header1".to_string(), "Header2".to_string()];
    let key = EndpointKey::new(request, headers_to_consider);
    let futures: Vec<_> = (0..100).map(|_| loader.load_one(key.clone())).collect();
    let _ = join_all(futures).await;
    assert_eq!(
      client.request_count.load(Ordering::SeqCst),
      1,
      "Only one request should be made for the same key"
    );
  }
  #[tokio::test]
  async fn test_load_function_many() {
    let client = MockHttpClient { request_count: Arc::new(AtomicUsize::new(0)) };

    let loader = HttpDataLoader { client: client.clone() };
    let loader = loader.to_async_data_loader_options(1, 1000);

    let request1 = reqwest::Request::new(reqwest::Method::GET, "http://example.com/1".parse().unwrap());
    let request2 = reqwest::Request::new(reqwest::Method::GET, "http://example.com/2".parse().unwrap());

    let headers_to_consider = vec!["Header1".to_string(), "Header2".to_string()];
    let key1 = EndpointKey::new(request1, headers_to_consider.clone());
    let key2 = EndpointKey::new(request2, headers_to_consider);
    let futures1 = (0..100).map(|_| loader.load_one(key1.clone()));
    let futures2 = (0..100).map(|_| loader.load_one(key2.clone()));
    let _ = join_all(futures1.chain(futures2)).await;
    assert_eq!(
      client.request_count.load(Ordering::SeqCst),
      2,
      "Only two requests should be made for two unique keys"
    );
  }
}
