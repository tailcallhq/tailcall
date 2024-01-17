use core::future::Future;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::ConstValue;
use futures_util::future::join_all;

use super::{Concurrency, Eval, EvaluationContext, EvaluationError, Expression, ResolverContextLike};

#[derive(Clone, Debug)]
pub enum List {
  Concat(Vec<Expression>),
}

impl Eval for List {
  fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrency,
  ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
    Box::pin(async move {
      match self {
        List::Concat(list) => join_all(list.iter().map(|expr| expr.eval(ctx, conc)))
          .await
          .into_iter()
          .try_fold(async_graphql::Value::List(vec![]), |acc, result| match (acc, result?) {
            (ConstValue::List(mut lhs), ConstValue::List(rhs)) => {
              lhs.extend(rhs);
              Ok(ConstValue::List(lhs))
            }
            _ => Err(EvaluationError::ExprEvalError("element is not a list".into()))?,
          }),
      }
    })
  }
}

impl<T, C> Eval<C> for T
where
  T: AsRef<[Expression]> + Send + Sync,
  C: FromIterator<ConstValue>,
{
  fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrency,
  ) -> Pin<Box<dyn Future<Output = Result<C>> + 'a + Send>> {
    Box::pin(async move {
      let future_iter = self.as_ref().iter().map(|expr| expr.eval(ctx, conc));
      match *conc {
        Concurrency::Parallel => join_all(future_iter).await.into_iter().collect::<Result<C>>(),
        Concurrency::Sequential => {
          let mut results = Vec::with_capacity(self.as_ref().len());
          for future in future_iter {
            results.push(future.await?);
          }
          Ok(results.into_iter().collect())
        }
      }
    })
  }
}
