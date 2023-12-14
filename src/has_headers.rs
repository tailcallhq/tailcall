use hyper::HeaderMap;

use crate::{lambda::{EvaluationContext, ResolverContextLike}, http::HttpClient};

pub trait HasHeaders {
  fn headers(&self) -> &HeaderMap;
}

impl<'a, Ctx: ResolverContextLike<'a>, Client: HttpClient> HasHeaders for EvaluationContext<'a, Ctx, Client> {
  fn headers(&self) -> &HeaderMap {
    self.headers()
  }
}
