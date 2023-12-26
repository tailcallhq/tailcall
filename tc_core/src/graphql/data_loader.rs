use std::collections::HashMap;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::futures_util::future::join_all;

use crate::config::Batch;
use crate::data_loader::{DataLoader, Loader};
use crate::http::{DataLoaderRequest, HttpClient, Response};

pub struct GraphqlDataLoader {
  pub client: Arc<dyn HttpClient>,
  pub batch: bool,
}

impl GraphqlDataLoader {
  pub fn new(client: Arc<dyn HttpClient>, batch: bool) -> Self {
    GraphqlDataLoader { client, batch }
  }

  pub fn to_data_loader(self, batch: Batch) -> DataLoader<DataLoaderRequest, GraphqlDataLoader> {
    DataLoader::new(self, tokio::spawn)
      .delay(Duration::from_millis(batch.delay as u64))
      .max_batch_size(batch.max_size)
  }
}

#[async_trait::async_trait]
impl Loader<DataLoaderRequest> for GraphqlDataLoader {
  type Value = Response;
  type Error = Arc<anyhow::Error>;

  #[allow(clippy::mutable_key_type)]
  async fn load(
    &self,
    keys: &[DataLoaderRequest],
  ) -> async_graphql::Result<HashMap<DataLoaderRequest, Self::Value>, Self::Error> {
    if self.batch {
      let batched_req = create_batched_request(keys);
      let result = self.client.execute(batched_req, None).await;
      let hashmap = extract_responses(result, keys);
      Ok(hashmap)
    } else {
      let results = keys.iter().map(|key| async {
        let result = self.client.execute(key.to_request(), None).await;
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

fn collect_request_bodies(dataloader_requests: &[DataLoaderRequest]) -> String {
  let batched_query = dataloader_requests
    .iter()
    .filter_map(|dataloader_req| {
      dataloader_req
        .body()
        .and_then(|body| body.as_bytes())
        // PERF: conversion from bytes to string with utf8 validation
        .and_then(|body| from_utf8(body).ok())
        .or(Some(""))
    })
    .collect::<Vec<_>>()
    .join(",");
  format!("[{}]", batched_query)
}

fn create_batched_request(dataloader_requests: &[DataLoaderRequest]) -> reqwest::Request {
  let batched_query = collect_request_bodies(dataloader_requests);

  let first_req = dataloader_requests.first().unwrap();
  let mut batched_req = first_req.to_request();
  batched_req.body_mut().replace(reqwest::Body::from(batched_query));
  batched_req
}

#[allow(clippy::mutable_key_type)]
fn extract_responses(
  result: Result<Response, anyhow::Error>,
  keys: &[DataLoaderRequest],
) -> HashMap<DataLoaderRequest, Response> {
  let mut hashmap = HashMap::new();
  if let Ok(res) = result {
    if let async_graphql_value::ConstValue::List(values) = res.body {
      for (i, request) in keys.iter().enumerate() {
        let value = values.get(i).unwrap_or(&async_graphql_value::ConstValue::Null);
        hashmap.insert(
          request.clone(),
          Response { status: res.status, headers: res.headers.clone(), body: value.clone() },
        );
      }
    }
  }
  hashmap
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;

  use reqwest::Url;

  use super::*;
  use crate::http::DataLoaderRequest;

  #[test]
  fn test_collect_request_bodies() {
    let url = Url::parse("http://example.com").unwrap();
    let mut request1 = reqwest::Request::new(reqwest::Method::GET, url.clone());
    request1.body_mut().replace(reqwest::Body::from("a".to_string()));
    let mut request2 = reqwest::Request::new(reqwest::Method::GET, url.clone());
    request2.body_mut().replace(reqwest::Body::from("b".to_string()));
    let mut request3 = reqwest::Request::new(reqwest::Method::GET, url.clone());
    request3.body_mut().replace(reqwest::Body::from("c".to_string()));

    let dl_req1 = DataLoaderRequest::new(request1, BTreeSet::new());
    let dl_req2 = DataLoaderRequest::new(request2, BTreeSet::new());
    let dl_req3 = DataLoaderRequest::new(request3, BTreeSet::new());

    let body = collect_request_bodies(&[dl_req1, dl_req2, dl_req3]);
    assert_eq!(body, "[a,b,c]");
  }
}
