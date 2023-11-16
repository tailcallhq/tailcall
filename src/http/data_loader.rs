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

fn get_body_value_single(body_value: HashMap<String, Vec<&ConstValue>>, id: &str) -> ConstValue {
  body_value
    .get(id)
    .and_then(|a| a.first().cloned().cloned())
    .unwrap_or(ConstValue::Null)
}

fn get_body_value_list(body_value: HashMap<String, Vec<&ConstValue>>, id: &str) -> ConstValue {
  ConstValue::List(
    body_value
      .get(id)
      .unwrap_or(&Vec::new())
      .iter()
      .map(|&o| o.to_owned())
      .collect::<Vec<_>>(),
  )
}

#[derive(Clone)]
pub struct HttpDataLoader {
  pub client: Arc<dyn HttpClient>,
  pub batched: Option<GroupBy>,
  #[allow(clippy::type_complexity)]
  pub body: fn(HashMap<String, Vec<&ConstValue>>, &str) -> ConstValue,
}
impl HttpDataLoader {
  pub fn new(client: Arc<dyn HttpClient>, batched: Option<GroupBy>, is_list: bool) -> Self {
    HttpDataLoader {
      client,
      batched,
      body: if is_list {
        get_body_value_list
      } else {
        get_body_value_single
      },
    }
  }

  pub fn to_data_loader(self, batch: Batch) -> DataLoader<HttpDataLoader, NoCache> {
    DataLoader::new(self, tokio::spawn)
      .delay(Duration::from_millis(batch.delay as u64))
      .max_batch_size(batch.max_size)
  }
}

#[async_trait::async_trait]
impl Loader<DataLoaderRequest> for HttpDataLoader {
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
      let path = &group_by.path();
      let body_value = res.body.group_by(path);

      for key in &keys {
        let req = key.to_request();
        let query_set: std::collections::HashMap<_, _> = req.url().query_pairs().collect();
        let id = query_set
          .get(group_by.key())
          .ok_or(anyhow::anyhow!("Unable to find key {} in query params", group_by.key()))?;
        hashmap.insert(key.clone(), res.clone().body((self.body)(body_value.clone(), id)));
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
  use std::collections::BTreeSet;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::sync::{Arc, Mutex};

  use super::*;
  use crate::http::DataLoaderRequest;

  #[derive(Clone)]
  struct MockHttpClient {
    // To keep track of number of times execute is called
    request_count: Arc<AtomicUsize>,
    // Keep track of the requests received
    requests: Arc<Mutex<Vec<reqwest::Request>>>,
    // Mock response
    response: Option<ConstValue>,
  }

  #[async_trait::async_trait]
  impl HttpClient for MockHttpClient {
    async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
      self.request_count.fetch_add(1, Ordering::SeqCst);
      self.requests.lock().unwrap().push(req.try_clone().unwrap());
      match &self.response {
        Some(value) => Ok(Response { body: value.clone(), ..Default::default() }),
        None => Ok(Response::default()),
      }
    }
  }
  #[tokio::test]
  async fn test_load_function() {
    let client = MockHttpClient {
      request_count: Arc::new(AtomicUsize::new(0)),
      requests: Arc::new(Mutex::new(Vec::new())),
      response: None,
    };

    let loader = HttpDataLoader::new(Arc::new(client.clone()), None, false);
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
    let client = MockHttpClient {
      request_count: Arc::new(AtomicUsize::new(0)),
      requests: Arc::new(Mutex::new(Vec::new())),
      response: None,
    };

    let loader = HttpDataLoader::new(Arc::new(client.clone()), None, false);
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

  #[tokio::test]
  async fn test_group_by() {
    let client = MockHttpClient {
      request_count: Arc::new(AtomicUsize::new(0)),
      requests: Arc::new(Mutex::new(Vec::new())),
      response: None,
    };

    let loader = HttpDataLoader::new(
      Arc::new(client.clone()),
      Some(GroupBy::new(vec!["userId".to_string()])),
      false,
    );
    let loader = loader.to_data_loader(Batch::default().delay(1));

    let request1 = reqwest::Request::new(reqwest::Method::GET, "http://example.com?userId=1".parse().unwrap());
    let request2 = reqwest::Request::new(reqwest::Method::GET, "http://example.com?userId=2".parse().unwrap());
    let headers = BTreeSet::new();
    let key1 = DataLoaderRequest::new(request1, headers.clone());
    let key2 = DataLoaderRequest::new(request2, headers);
    let future1 = loader.load_one(key1);
    let future2 = loader.load_one(key2);
    let _ = join_all([future1, future2]).await;
    assert_eq!(
      client.request_count.load(Ordering::SeqCst),
      1,
      "Only one request should be sent if grouped"
    );
    assert_eq!(
      client.requests.lock().unwrap().first().unwrap().url().to_string(),
      "http://example.com/?userId=1&userId=2"
    );
  }

  #[tokio::test]
  async fn test_batch_size_with_group_by() {
    let client = MockHttpClient {
      request_count: Arc::new(AtomicUsize::new(0)),
      requests: Arc::new(Mutex::new(Vec::new())),
      response: None,
    };

    let loader = HttpDataLoader::new(
      Arc::new(client.clone()),
      Some(GroupBy::new(vec!["userId".to_string()])),
      false,
    );
    let loader = loader.to_data_loader(Batch::default().delay(1).max_size(1));

    let request1 = reqwest::Request::new(reqwest::Method::GET, "http://example.com?userId=1".parse().unwrap());
    let request2 = reqwest::Request::new(reqwest::Method::GET, "http://example.com?userId=2".parse().unwrap());
    let headers = BTreeSet::new();
    let key1 = DataLoaderRequest::new(request1, headers.clone());
    let key2 = DataLoaderRequest::new(request2, headers);
    let future1 = loader.load_one(key1);
    let future2 = loader.load_one(key2);
    let _ = join_all([future1, future2]).await;
    assert_eq!(
      client.request_count.load(Ordering::SeqCst),
      2,
      "Two requests should be sent if batch size = 1"
    );
    assert_eq!(
      client.requests.lock().unwrap().first().unwrap().url().to_string(),
      "http://example.com/?userId=1"
    );
    assert_eq!(
      client.requests.lock().unwrap().get(1).unwrap().url().to_string(),
      "http://example.com/?userId=2"
    );
  }
}
