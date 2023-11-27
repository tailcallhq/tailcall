use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_graphql::futures_util::future::join_all;

use crate::config::Batch;
use crate::http::{DataLoaderRequest, HttpClient, HttpDataLoader, Response};

#[derive(Clone)]
pub struct MockHttpClient {
  // To keep track of the number of times execute is called
  pub request_count: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
  async fn execute(&self, _req: reqwest::Request) -> anyhow::Result<Response> {
    self.request_count.fetch_add(1, Ordering::SeqCst);
    // You can mock the actual response as per your need
    Ok(Response::default())
  }
}

pub async fn run_data_loader_benchmark(client: Arc<MockHttpClient>) {
  let loader = HttpDataLoader::new(client.clone(), None, false);
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
    "Only one request should be made for the same key"
  );
}
