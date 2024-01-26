use crate::helpers;
use crate::json::JsonLike;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU64;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::ConstValue;

use super::{Concurrent, Eval, EvaluationContext, Expression, ResolverContextLike};


#[derive(Clone, Debug)]
pub struct Cache {
  hasher: DefaultHasher,
  max_age: NonZeroU64,
  source: Box<Expression>,
}

impl Cache {
  pub fn new(hasher: DefaultHasher, max_age: NonZeroU64, source: Box<Expression>) -> Self {
    Self { hasher, max_age, source }
  }

  pub fn hasher(&self) -> &DefaultHasher {
    &self.hasher
  }

  pub fn max_age(&self) -> NonZeroU64 {
    self.max_age
  }

  pub fn source(&self) -> &Expression {
    &self.source
  }
}

impl Eval for Cache {
  fn eval<'a, Ctx: super::ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a super::EvaluationContext<'a, Ctx>,
    conc: &'a super::Concurrent,
  ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
    Box::pin(async move {
      // let ttl_and_key = Some((self.max_age, get_cache_key(&ctx, &self.hasher)?));
      // let cache = ctx.req_ctx.cache.clone();
      // let source = self.source.eval(ctx, conc).await?;
      // let value = cache.get_or_insert_with(ttl_and_key, || Ok(source)).await?;
      // Ok(value)

          if let Some(key) = get_cache_key(ctx, self.hasher.clone()) {
            if let Some(cache_value) = ctx.req_ctx.cache_get(&key).await {
              Ok(cache_value.to_owned())
            } else {
              let result = self.source.eval(ctx, conc).await;
              if let Ok(val) = &result {
                ctx.req_ctx.cache_insert(key, val.clone(), self.max_age);
              }
              result
            }
          } else {
            self.source.eval(ctx, conc).await
          }
    })
  }
}

fn get_cache_key<'a, H: Hasher + Clone>(
  ctx: &'a EvaluationContext<'a, impl ResolverContextLike<'a>>,
  mut hasher: H,
) -> Option<u64> {
  // Hash on parent value
  if let Some(const_value) = ctx
    .graphql_ctx
    .value()
    // TODO: handle _id, id, or any field that has @key on it.
    .filter(|value| value != &&ConstValue::Null)
    .map(|data| data.get_key("id"))
  {
    // Hash on parent's id only?
    helpers::value::hash(const_value?, &mut hasher)
  }

  let key = ctx.graphql_ctx.args().map(|value_map| {
    value_map
      .iter()
      .map(|(key, value)| {
        let mut hasher = hasher.clone();
        key.hash(&mut hasher);
        helpers::value::hash(value, &mut hasher);
        hasher.finish()
      })
      .fold(hasher.finish(), |acc, val| acc ^ val)
  });
  key
}
