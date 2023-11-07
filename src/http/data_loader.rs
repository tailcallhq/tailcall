use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::dataloader::{DataLoader, Loader, NoCache};
use async_graphql_value::ConstValue;
use hashbrown::HashMap as BrownHashMap;

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

fn sorted_key_loader(keys: &[DataLoaderRequest]) -> Vec<DataLoaderRequest> {
  let mut keys: Vec<DataLoaderRequest> = keys.to_vec();
  keys.sort_by(|a, b| a.to_request().url().cmp(b.to_request().url()));
  keys
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
      // store the preused length of our keys.
      let original_key_length = keys.len();

      let mut request = keys[0].to_request();

      let first_url = request.url_mut();

      let urls = keys[1..].iter().map(|key: &DataLoaderRequest| {
        let key_request_result = key.to_request();
        tokio::task::spawn(async move {
          let url = key_request_result.url().to_owned();
          url
        })
      });

      for url_handle in urls {
        //TODO: replace this unsafe unwrap, and handle the error
        let url = url_handle.await.unwrap();
        first_url.query_pairs_mut().extend_pairs(url.query_pairs());
      }

      let res = self.client.execute(request).await?;

      #[allow(clippy::mutable_key_type)]
      let mut hashmap: HashMap<DataLoaderRequest, Response> = HashMap::with_capacity(original_key_length);

      //TODO: figure out something lifetime elision raised here due to 'life0 mismatch
      let group_key = group_by.clone().key().to_string();
      let path = group_by.path();
      let body_value = res.body.group_by(&path[..]);

      let mut res_handle = Vec::with_capacity(original_key_length);

      for key in keys.iter() {
        let req = key.to_request();
        let key_cloned = key.clone();
        let group_key_clone = group_key.clone();

        res_handle.push(tokio::task::spawn(async move {
          let query_set: HashMap<_, _> = req.url().query_pairs().collect();
          //TODO: replace this unsafe unwrap, and handle the error.

          let id = query_set
            .get(group_key_clone.as_str())
            .ok_or(anyhow::anyhow!(
              "Unable to find key {} in query params",
              group_key_clone
            ))
            .unwrap()
            .deref()
            .to_owned();
          (key_cloned, id)
        }))
      }

      for set in res_handle {
        //TODO: replace this unsafe unwrap, and handle the error.
        let (key, id) = set.await.unwrap();
        hashmap.insert(
          key.clone(),
          res.clone().body(
            body_value
              .get(&id)
              .and_then(|a| a.first().cloned().cloned())
              .unwrap_or(ConstValue::Null),
          ),
        );
      }
      Ok(hashmap)
    } else {
      let results = keys.iter().map(|key| {
        let cloned_client = self.client.clone();
        let key_request = key.to_request();
        let key_cloned = key.clone();
        tokio::task::spawn(async move {
          let query_result = cloned_client.execute(key_request).await;
          (key_cloned, query_result)
        })
      });

      let mut hashmap = HashMap::new();

      for val in results {
        //TODO: replace this unsafe unwrap
        let (key, value) = val.await.unwrap();
        hashmap.insert(key, value?);
      }

      Ok(HashMap::from_iter(hashmap))
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;
  use std::sync::atomic::{AtomicUsize, Ordering};

  use async_graphql::futures_util::future::join_all;

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
    let headers_to_consider = BTreeSet::from(["Header1".to_string(), "Header2".to_string()]);
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

    let headers_to_consider = BTreeSet::from(["Header1".to_string(), "Header2".to_string()]);
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
