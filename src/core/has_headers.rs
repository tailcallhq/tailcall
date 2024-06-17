use hyper::HeaderMap;

use crate::core::ir::{EvalContext, ResolverContextLike};

pub trait HasHeaders {
    fn headers(&self) -> &HeaderMap;
}

impl<'a, Ctx: ResolverContextLike> HasHeaders for EvalContext<'a, Ctx> {
    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
}
