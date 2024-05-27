use hyper::header::HeaderMap;

use crate::core::ir::{EvaluationContext, ResolverContextLike};

pub trait HasHeaders {
    // TODO: Try converting headers() to reqwest
    fn headers(&self) -> &HeaderMap;
}

impl<'a, Ctx: ResolverContextLike<'a>> HasHeaders for EvaluationContext<'a, Ctx> {
    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
}
