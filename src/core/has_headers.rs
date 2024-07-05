use hyper::header::HeaderMap;

use crate::core::ir::{EvalContext, ResolverContextLike};

pub trait HasHeaders {
    // TODO: Try converting headers() to reqwest
    fn headers(&self) -> &HeaderMap;
}

impl<'a, Ctx: ResolverContextLike> HasHeaders for EvalContext<'a, Ctx> {
    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
}
