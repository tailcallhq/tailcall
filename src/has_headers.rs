use hyper::HeaderMap;

use crate::lambda::{EvaluationContext, GraphqlContext};

pub trait HasHeaders {
  fn headers(&self) -> &HeaderMap;
}

impl<'a, Ctx: GraphqlContext<'a>> HasHeaders for EvaluationContext<'a, Ctx> {
  fn headers(&self) -> &HeaderMap {
    self.headers()
  }
}
