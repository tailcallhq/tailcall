use std::num::NonZeroU64;


use super::{IR};

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct IoId(u64);
impl IoId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
pub trait CacheKey<Ctx> {
    fn cache_key(&self, ctx: &Ctx) -> Option<IoId>;
}

#[derive(Clone, Debug)]
pub struct Cache {
    pub max_age: NonZeroU64,
    pub expr: Box<IR>,
}

impl Cache {
    ///
    /// Wraps an expression with the cache primitive.
    /// Performance DFS on the cache on the expression and identifies all the IO
    /// nodes. Then wraps each IO node with the cache primitive.
    pub fn wrap(max_age: NonZeroU64, expr: IR) -> IR {
        expr.modify(move |expr| match expr {
            IR::IO(_) => Some(IR::Cache(Cache { max_age, expr: Box::new(expr.clone()) })),
            _ => None,
        })
    }
}
