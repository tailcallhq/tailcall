use http::header::HeaderMap;

use crate::core::ir::{EvalContext, ResolverContextLike};

pub trait HasHeaders {
    fn headers(&self) -> &HeaderMap;
}

impl<Ctx: ResolverContextLike> HasHeaders for EvalContext<'_, Ctx> {
    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
}
