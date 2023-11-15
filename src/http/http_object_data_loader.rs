use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::dataloader::{DataLoader, Loader, NoCache};
use async_graphql_value::ConstValue;

use crate::config::group_by::GroupBy;
use crate::config::Batch;
use crate::http::data_loader::{get_result_map, LoaderOptions};
use crate::http::{DataLoaderRequest, HttpClient, Response};

#[derive(Clone)]
pub struct HttpObjectDataLoader {
  pub client: Arc<dyn HttpClient>,
  pub batched: Option<GroupBy>,
}

impl HttpObjectDataLoader {
  pub fn new(client: Arc<dyn HttpClient>, batched: Option<GroupBy>) -> Self {
    HttpObjectDataLoader { client, batched }
  }

  pub fn to_data_loader(self, batch: Batch) -> DataLoader<HttpObjectDataLoader, NoCache> {
    DataLoader::new(self, tokio::spawn)
      .delay(Duration::from_millis(batch.delay as u64))
      .max_batch_size(batch.max_size)
  }

  pub fn get_body_value(body_value_map: HashMap<String, Vec<&ConstValue>>, id: &str) -> ConstValue {
    body_value_map
      .get(id)
      .and_then(|a| a.first().cloned().cloned())
      .unwrap_or(ConstValue::Null)
  }
}

#[async_trait::async_trait]
impl Loader<DataLoaderRequest> for HttpObjectDataLoader {
  type Value = Response;
  type Error = Arc<anyhow::Error>;

  async fn load(
    &self,
    keys: &[DataLoaderRequest],
  ) -> async_graphql::Result<HashMap<DataLoaderRequest, Self::Value>, Self::Error> {
    Ok(
      get_result_map(
        LoaderOptions { batched: self.batched.clone(), client: self.client.clone() },
        keys,
        HttpObjectDataLoader::get_body_value,
      )
      .await?,
    )
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::sync::{Arc, Mutex};

  use async_graphql::futures_util::future::join_all;
  use async_graphql::Name;
  use indexmap::IndexMap;

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

    let loader = HttpObjectDataLoader { client: Arc::new(client.clone()), batched: None };
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

    let loader = HttpObjectDataLoader { client: Arc::new(client.clone()), batched: None };
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

    let loader = HttpObjectDataLoader::new(Arc::new(client.clone()), Some(GroupBy::new(vec!["userId".to_string()])));
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

    let loader = HttpObjectDataLoader::new(Arc::new(client.clone()), Some(GroupBy::new(vec!["userId".to_string()])));
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

  #[tokio::test]
  async fn test_batch_object_field() {
    let mut post1 = IndexMap::<async_graphql::Name, ConstValue>::new();
    post1.insert(Name::new("userId"), ConstValue::String("1".into()));
    post1.insert(Name::new("title"), ConstValue::String("abc1".into()));
    let mut post2 = IndexMap::<async_graphql::Name, ConstValue>::new();
    post2.insert(Name::new("userId"), ConstValue::String("2".into()));
    post2.insert(Name::new("title"), ConstValue::String("abc2".into()));

    let mock_response = ConstValue::List(vec![ConstValue::Object(post1), ConstValue::Object(post2)]);

    let client = MockHttpClient {
      request_count: Arc::new(AtomicUsize::new(0)),
      requests: Arc::new(Mutex::new(Vec::new())),
      response: Some(mock_response),
    };

    let loader = HttpObjectDataLoader::new(Arc::new(client.clone()), Some(GroupBy::new(vec!["userId".to_string()])));
    let loader = loader.to_data_loader(Batch::default().delay(1));

    let request1 = reqwest::Request::new(reqwest::Method::GET, "http://example.com?userId=1".parse().unwrap());
    let request2 = reqwest::Request::new(reqwest::Method::GET, "http://example.com?userId=2".parse().unwrap());
    let headers = BTreeSet::new();
    let key1 = DataLoaderRequest::new(request1, headers.clone());
    let key2 = DataLoaderRequest::new(request2, headers);
    let future1 = loader.load_one(key1);
    let future2 = loader.load_one(key2);
    let response = join_all([future1, future2]).await;
    assert!(matches!(
      response
        .first()
        .unwrap()
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap()
        .body
        .clone(),
      ConstValue::Object(..)
    ));
  }
}
