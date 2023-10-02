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
    for (name, value) in self
      .headers()
      .iter()
      .filter(|(name, _)| self.1.contains(&name.as_str().to_string()))
    {
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

#[cfg(test)]
mod tests {
  use hyper::header::HeaderValue;
  use hyper::http::HeaderName;

  use super::*;

  fn create_request_with_headers(url: &str, headers: Vec<(&str, &str)>) -> reqwest::Request {
    let mut req = reqwest::Request::new(reqwest::Method::GET, url.parse().unwrap());
    for (name, value) in headers {
      req.headers_mut().insert(
        name.parse::<HeaderName>().unwrap(),
        value.parse::<HeaderValue>().unwrap(),
      );
    }
    req
  }

  fn create_endpoint_key(url: &str, headers: Vec<(&str, &str)>, hash_key_headers: Vec<String>) -> EndpointKey {
    EndpointKey::new(create_request_with_headers(url, headers), hash_key_headers)
  }

  #[test]
  fn test_hash_endpoint_key() {
    let endpoint_key_1 = create_endpoint_key("http://localhost:8080", vec![], vec![]);
    let endpoint_key_2 = create_endpoint_key("http://localhost:8080", vec![], vec![]);
    assert_eq!(endpoint_key_1, endpoint_key_2);
  }

  #[test]
  fn test_with_endpoint_key_with_headers() {
    let endpoint_key_1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2")],
      vec!["a".to_string(), "b".to_string()],
    );
    let endpoint_key_2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2"), ("c", "3")],
      vec!["a".to_string(), "b".to_string()],
    );
    assert_eq!(endpoint_key_1, endpoint_key_2);
  }

  #[test]
  fn test_with_endpoint_key_with_headers_ne() {
    let endpoint_key_1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2"), ("c", "4")],
      vec!["a".to_string(), "b".to_string(), "c".to_string()],
    );
    let endpoint_key_2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2"), ("c", "3")],
      vec!["a".to_string(), "b".to_string(), "c".to_string()],
    );
    assert_ne!(endpoint_key_1, endpoint_key_2);
  }
  #[test]
  fn test_different_http_methods() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![], vec![]);
    let req = reqwest::Request::new(reqwest::Method::POST, "http://localhost:8080".parse().unwrap());
    let key2 = EndpointKey::new(req, vec![]);
    assert_ne!(key1, key2);
  }

  #[test]
  fn test_different_urls() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![], vec![]);
    let key2 = create_endpoint_key("http://example.com:8080", vec![], vec![]);
    assert_ne!(key1, key2);
  }

  #[test]
  fn test_mismatched_header_names() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![("a", "1")], vec!["a".to_string()]);
    let key2 = create_endpoint_key("http://localhost:8080", vec![("b", "1")], vec!["b".to_string()]);
    assert_ne!(key1, key2);
  }

  #[test]
  fn test_mismatched_header_values() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![("a", "1")], vec!["a".to_string()]);
    let key2 = create_endpoint_key("http://localhost:8080", vec![("a", "2")], vec!["a".to_string()]);
    assert_ne!(key1, key2);
  }

  #[test]
  fn test_differing_number_of_headers() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![("a", "1")], vec!["a".to_string()]);
    let key2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2")],
      vec!["a".to_string(), "b".to_string()],
    );
    assert_ne!(key1, key2);
  }
  #[test]
  fn test_clone_trait() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![("a", "1")], vec!["a".to_string()]);
    let key2 = key1.clone();

    // The cloned key should be equal to the original
    assert_eq!(key1, key2);

    // The memory addresses of the two keys should be different
    assert!(!std::ptr::eq(&key1, &key2));
  }

  #[test]
  fn test_partial_eq_trait() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![("a", "1")], vec!["a".to_string()]);
    let key2 = create_endpoint_key("http://localhost:8080", vec![("a", "1")], vec!["a".to_string()]);

    // Both keys have the same data, so they should be equal
    assert_eq!(key1, key2);
  }

  #[test]
  fn test_partial_eq_not_equal() {
    let key1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "1")],
      vec!["a".to_string(), "b".to_string()],
    );
    let key2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "1")],
      vec!["a".to_string()],
    );

    assert_ne!(key1, key2);
  }
}
