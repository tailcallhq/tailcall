use reqwest::header::HeaderMap;

use crate::lambda::{EvaluationContext, ResolverContextLike};

pub trait HasHeaders {
    fn headers(&self) -> &HeaderMap;
}

impl<'a, Ctx: ResolverContextLike<'a>> HasHeaders for EvaluationContext<'a, Ctx> {
    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
}
