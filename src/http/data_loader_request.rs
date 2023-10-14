use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct DataLoaderRequest(reqwest::Request, BTreeSet<String>);

impl DataLoaderRequest {
  pub fn new(req: reqwest::Request, headers: BTreeSet<String>) -> Self {
    DataLoaderRequest(req, headers)
  }
  pub fn to_request(&self) -> reqwest::Request {
    self.clone().0
  }
  pub fn headers(&self) -> &BTreeSet<String> {
    &self.1
  }
}
impl Hash for DataLoaderRequest {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.url().hash(state);
    if let Some(body) = self.0.body() {
      if let Some(bytes) = body.as_bytes() {
        if let Ok(body_str) = std::str::from_utf8(bytes) {
          if !body_str.is_empty() {
            body_str.to_string().hash(state);
          }
        }
      }
    }
    for name in &self.1 {
      if let Some(value) = self.0.headers().get(name) {
        name.hash(state);
        value.hash(state);
      }
    }
  }
}

impl PartialEq for DataLoaderRequest {
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

impl Clone for DataLoaderRequest {
  fn clone(&self) -> Self {
    let req = self.0.try_clone().unwrap();
    // let mut req = reqwest::Request::new(reqwest::Method::GET, self.0.url().clone());
    // req.headers_mut().extend(self.0.headers().clone());

    DataLoaderRequest(req, self.1.clone())
  }
}

impl Eq for DataLoaderRequest {}

#[cfg(test)]
mod tests {
  use hyper::header::{HeaderName, HeaderValue};

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

  fn create_endpoint_key(
    url: &str,
    headers: Vec<(&str, &str)>,
    hash_key_headers: BTreeSet<String>,
  ) -> DataLoaderRequest {
    DataLoaderRequest::new(create_request_with_headers(url, headers), hash_key_headers)
  }

  #[test]
  fn test_hash_endpoint_key() {
    let endpoint_key_1 = create_endpoint_key("http://localhost:8080", vec![], BTreeSet::new());
    let endpoint_key_2 = create_endpoint_key("http://localhost:8080", vec![], BTreeSet::new());
    assert_eq!(endpoint_key_1, endpoint_key_2);
  }

  #[test]
  fn test_with_endpoint_key_with_headers() {
    let endpoint_key_1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2")],
      BTreeSet::from(["a".to_string(), "b".to_string()]),
    );
    let endpoint_key_2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2"), ("c", "3")],
      BTreeSet::from(["a".to_string(), "b".to_string()]),
    );
    assert_eq!(endpoint_key_1, endpoint_key_2);
  }

  #[test]
  fn test_with_endpoint_key_with_headers_ne() {
    let endpoint_key_1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2"), ("c", "4")],
      BTreeSet::from(["a".to_string(), "b".to_string(), "c".to_string()]),
    );
    let endpoint_key_2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2"), ("c", "3")],
      BTreeSet::from(["a".to_string(), "b".to_string(), "c".to_string()]),
    );
    assert_ne!(endpoint_key_1, endpoint_key_2);
  }
  #[test]
  fn test_different_http_methods() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![], BTreeSet::new());
    let req = reqwest::Request::new(reqwest::Method::POST, "http://localhost:8080".parse().unwrap());
    let key2 = DataLoaderRequest::new(req, BTreeSet::new());
    assert_eq!(key1, key2);
  }

  #[test]
  fn test_different_urls() {
    let key1 = create_endpoint_key("http://localhost:8080", vec![], BTreeSet::new());
    let key2 = create_endpoint_key("http://example.com:8080", vec![], BTreeSet::new());
    assert_ne!(key1, key2);
  }

  #[test]
  fn test_mismatched_header_names() {
    let key1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1")],
      BTreeSet::from(["a".to_string()]),
    );
    let key2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("b", "1")],
      BTreeSet::from(["b".to_string()]),
    );
    assert_ne!(key1, key2);
  }

  #[test]
  fn test_mismatched_header_values() {
    let key1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1")],
      BTreeSet::from(["a".to_string()]),
    );
    let key2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "2")],
      BTreeSet::from(["a".to_string()]),
    );
    assert_ne!(key1, key2);
  }

  #[test]
  fn test_differing_number_of_headers() {
    let key1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1")],
      BTreeSet::from(["a".to_string()]),
    );
    let key2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "2")],
      BTreeSet::from(["a".to_string(), "b".to_string()]),
    );
    assert_ne!(key1, key2);
  }
  #[test]
  fn test_clone_trait() {
    let key1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1")],
      BTreeSet::from(["a".to_string()]),
    );
    let key2 = key1.clone();

    // The cloned key should be equal to the original
    assert_eq!(key1, key2);
  }

  #[test]
  fn test_partial_eq_trait() {
    let key1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1")],
      BTreeSet::from(["a".to_string()]),
    );
    let key2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1")],
      BTreeSet::from(["a".to_string()]),
    );

    // Both keys have the same data, so they should be equal
    assert_eq!(key1, key2);
  }

  #[test]
  fn test_partial_eq_not_equal() {
    let key1 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "1")],
      BTreeSet::from(["a".to_string(), "b".to_string()]),
    );
    let key2 = create_endpoint_key(
      "http://localhost:8080",
      vec![("a", "1"), ("b", "1")],
      BTreeSet::from(["a".to_string()]),
    );

    assert_ne!(key1, key2);
  }
}
