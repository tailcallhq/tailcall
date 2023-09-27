use hyper::HeaderMap;

use crate::lambda::EvaluationContext;

pub trait HasHeaders {
  fn headers(&self) -> &HeaderMap;
}

impl HasHeaders for EvaluationContext<'_> {
  fn headers(&self) -> &HeaderMap {
    self.headers()
  }
}
