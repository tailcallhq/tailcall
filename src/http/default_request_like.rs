use http_cache_semantics::RequestLike;

pub struct DefaultRequestLike {
  inner_headers: hyper::HeaderMap,
}

impl RequestLike for DefaultRequestLike {
  fn uri(&self) -> hyper::Uri {
    hyper::Uri::default()
  }

  fn is_same_uri(&self, _other: &hyper::Uri) -> bool {
    true
  }

  fn method(&self) -> &hyper::Method {
    &hyper::Method::GET
  }

  fn headers(&self) -> &hyper::HeaderMap {
    &self.inner_headers
  }
}

impl DefaultRequestLike {
  pub fn upcast(&self) -> &DefaultRequestLike {
    self
  }
}

impl Default for DefaultRequestLike {
  fn default() -> Self {
    DefaultRequestLike { inner_headers: hyper::HeaderMap::new() }
  }
}
