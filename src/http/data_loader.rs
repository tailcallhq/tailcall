use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use async_graphql::async_trait;
use async_graphql::dataloader::{DataLoader, Loader, NoCache};
use async_graphql_value::ConstValue;
use dashmap::DashMap;
use tokio::sync::RwLock;

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

// fn sorted_key_loader(keys: &[DataLoaderRequest]) -> Vec<DataLoaderRequest> {
//   let mut keys: Vec<DataLoaderRequest> = keys.to_vec();
//   keys.sort_by(|a, b| a.to_request().url().cmp(b.to_request().url()));
//   keys
// }

fn return_request(keys: &[DataLoaderRequest]) -> Vec<reqwest::Request> {
  keys.iter().map(|key| key.to_request()).collect()
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

      /*
      From the &[DataLoaderRequest]
      Create a Vec<Request> of same length
      */

      // store the preused length of our keys.
      let original_key_length = keys.len();

      log::info!("Original key length: {}", original_key_length);
      println!("Original key length: {}", original_key_length);

      let request_keys = return_request(keys);

      //TODO: sorting is yet to be tested 
      // request_keys.sort_by(|a, b| a.url().cmp(b.url()));
      
      let mut request = request_keys[0].try_clone()
        .unwrap();

      // the serializer by default implements the auto not marker traits for the types
      let first_url = request
        .url_mut();

      // perform this under single iteration on the current thread runtime !
      for key in request_keys[1..].iter() {
        let key_url = key.url();
        let url = key_url.query_pairs();
        //TODO: The following method `query_pairs_mut()` is a expensive operation, requires a better approach if possible.
        first_url.query_pairs_mut().extend_pairs(url);
      }

      let client_res = self.client.execute(request).await?;
    
      #[allow(clippy::mutable_key_type)]
      let hashmap: Arc<DashMap<DataLoaderRequest, Response>> = Arc::new(DashMap::with_capacity(original_key_length));

      let path = group_by.path();
      let group_key = group_by.key();

      let body_value = client_res.body.group_by(&path[..]);

      let mut res_handle = Vec::with_capacity(original_key_length);

      //TODO: This will fail if sorting procedure is not done. 
      for (req, dataloader) in request_keys.into_iter().zip(keys) {
        
        let key_cloned = dataloader.to_owned();
        let group_key_clone = group_key;
        let query_url = req.url();

        let query_set: HashMap<Cow<'_, str>, Cow<'_, str>> = query_url.query_pairs().collect();

        //TODO: replace this unsafe unwrap, and handle the error.
        let id = query_set
          .get(group_key_clone)
          .ok_or(anyhow::anyhow!(
            "Unable to find key {} in query params",
            group_key_clone
          ))
          .unwrap()
          .deref()
          .to_owned();

        let body_key = body_value
          .get(&id)
          .and_then(|a| a.first().cloned().cloned())
          .unwrap_or(ConstValue::Null);
        
        let client_key = client_res.clone().body(body_key);
        
        let hm_handle = Arc::clone(&hashmap);

        let sub_task = tokio::task::spawn(async move {

          hm_handle.insert(
            key_cloned, 
            client_key
          )

        });
        res_handle.push(sub_task);
      }

      for task in res_handle { 
        if let Err(e) = task.await {
          return Err(Arc::new(anyhow!("Task failed: {:?}", e)));
        };
      }
      //todo: replace this unsafe unwrap
      let hashmap = Arc::into_inner(hashmap).unwrap();
    
      Ok(HashMap::from_iter(hashmap))
    
    } else {
      let results = keys.into_iter().map(|key| {
        let cloned_client = self.client.clone();
        let key_request = key.to_request();
        let key_cloned = key.to_owned();
        (key_cloned, tokio::task::spawn(async move {
          let query_result = cloned_client.execute(key_request).await;
           query_result
        }))
      });

      let mut hashmap = HashMap::new();

      for (key, task) in results {
        //TODO: replace this unsafe unwrap
        let value = task.await.unwrap();
        hashmap.insert(key, value?);
      }

      Ok(hashmap)
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
