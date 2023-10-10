use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::dataloader::{DataLoader, Loader, NoCache};
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;

use crate::config::group_by::GroupBy;
use crate::config::Batch;
use crate::http::{DataLoaderRequest, HttpClient, Response};
use crate::json::JsonLike;

#[derive(Default, Clone, Debug)]
pub struct HttpDataLoader<C>
where
  C: HttpClient + Send + Sync + 'static + Clone,
{
  pub client: C,
  pub batched: Option<GroupBy>,
}
impl<C: HttpClient + Send + Sync + 'static + Clone> HttpDataLoader<C> {
  pub fn new(client: C, batched: Option<GroupBy>) -> Self {
    HttpDataLoader { client, batched }
  }

  pub fn to_data_loader(self, batch: Batch) -> DataLoader<HttpDataLoader<C>, NoCache> {
    DataLoader::new(self, tokio::spawn)
      .delay(Duration::from_millis(batch.delay as u64))
      .max_batch_size(batch.max_size)
  }
}

#[async_trait::async_trait]
impl<C: HttpClient + Send + Sync + 'static + Clone> Loader<DataLoaderRequest> for HttpDataLoader<C> {
  type Value = Response;
  type Error = Arc<anyhow::Error>;

  async fn load(
    &self,
    keys: &[DataLoaderRequest],
  ) -> async_graphql::Result<HashMap<DataLoaderRequest, Self::Value>, Self::Error> {
    if let Some(group_by) = self.batched.clone() {
      let mut keys = keys.to_vec();
      keys.sort_by(|a, b| a.to_request().url().cmp(b.to_request().url()));

      let mut request = keys[0].to_request();
      let first_url = request.url_mut();

      for key in &keys[1..] {
        let request = key.to_request();
        let url = request.url();
        first_url.query_pairs_mut().extend_pairs(url.query_pairs());
      }

      let res = self.client.execute(request).await?;
      #[allow(clippy::mutable_key_type)]
      let mut hashmap: HashMap<DataLoaderRequest, Response> = HashMap::with_capacity(keys.len());
      let body_value = res.body.group_by(group_by.path());

      for key in &keys {
        let req = key.to_request();
        let query_set: std::collections::HashMap<_, _> = req.url().query_pairs().collect();
        let id = query_set
          .get(group_by.key().clone().as_str())
          .ok_or(anyhow::anyhow!("Unable to find key {} in query params", group_by.key()))?;
        hashmap.insert(
          key.clone(),
          res.clone().body(
            body_value
              .get(id.as_ref())
              .and_then(|a| a.first().cloned().cloned())
              .unwrap_or(ConstValue::Null),
          ),
        );
      }

      Ok(hashmap)
    } else {
      let results = keys.iter().map(|key| async {
        let result = self.client.execute(key.to_request()).await;
        (key.clone(), result)
      });

      let results = join_all(results).await;

      #[allow(clippy::mutable_key_type)]
      let mut hashmap = HashMap::new();
      for (key, value) in results {
        hashmap.insert(key, value?);
      }

      Ok(hashmap)
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicUsize, Ordering};

  use super::*;
  use crate::http::DataLoaderRequest;

  #[derive(Clone)]
  struct MockHttpClient {
    // To keep track of number of times execute is called
    request_count: Arc<AtomicUsize>,
  }

  #[async_trait::async_trait]
  impl HttpClient for MockHttpClient {
    async fn execute(&self, _req: reqwest::Request) -> anyhow::Result<Response> {
      self.request_count.fetch_add(1, Ordering::SeqCst);
      // You can mock the actual response as per your need
      Ok(Response::default())
    }
  }
  #[tokio::test]
  async fn test_load_function() {
    let client = MockHttpClient { request_count: Arc::new(AtomicUsize::new(0)) };

    let loader = HttpDataLoader { client: client.clone(), batched: None };
    let loader = loader.to_data_loader(Batch::default().delay(1));

    let request = reqwest::Request::new(reqwest::Method::GET, "http://example.com".parse().unwrap());
    let headers_to_consider = vec!["Header1".to_string(), "Header2".to_string()];
    let key = DataLoaderRequest::new(request, headers_to_consider);
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

    let loader = HttpDataLoader { client: client.clone(), batched: None };
    let loader = loader.to_data_loader(Batch::default().delay(1));

    let request1 = reqwest::Request::new(reqwest::Method::GET, "http://example.com/1".parse().unwrap());
    let request2 = reqwest::Request::new(reqwest::Method::GET, "http://example.com/2".parse().unwrap());

    let headers_to_consider = vec!["Header1".to_string(), "Header2".to_string()];
    let key1 = DataLoaderRequest::new(request1, headers_to_consider.clone());
    let key2 = DataLoaderRequest::new(request2, headers_to_consider);
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
