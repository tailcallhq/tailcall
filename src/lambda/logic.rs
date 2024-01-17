use anyhow::Result;
use async_graphql_value::ConstValue;
use futures_util::future::join_all;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;

use super::helpers::{is_empty, is_truthy, is_truthy_ref};
use super::{Concurrency, Eval, EvaluationContext, Expression, ResolverContextLike};

#[derive(Clone, Debug)]
pub enum Logic {
  If {
    cond: Box<Expression>,
    then: Box<Expression>,
    els: Box<Expression>,
  },
  And(Vec<Expression>),
  Or(Vec<Expression>),
  Cond(Box<Expression>, Vec<(Box<Expression>, Box<Expression>)>),
  DefaultTo(Box<Expression>, Box<Expression>),
  IsEmpty(Box<Expression>),
  Not(Box<Expression>),
}

impl Eval for Logic {
  async fn async_eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrency,
  ) -> Result<async_graphql::Value> {
    Ok(match self {
      Logic::Or(list) => {
        let future_iter = list.iter().map(|expr| expr.eval(ctx, conc));

        match *conc {
          Concurrency::Parallel => {
            let mut futures: FuturesUnordered<_> = future_iter.collect();
            let mut output = false;

            while let Some(result) = futures.next().await {
              let result: Result<ConstValue> = result;
              if is_truthy(result?) {
                output = true;
                break;
              }
            }
            output
          }
          Concurrency::Sequential => {
            let mut output = false;
            for future in future_iter.into_iter() {
              if is_truthy(future.await?) {
                output = true;
                break;
              }
            }
            output
          }
        }
        .into()
      }
      Logic::Cond(default, list) => match *conc {
        Concurrency::Sequential => {
          let mut result = None;
          for (cond, expr) in list.iter() {
            if is_truthy(cond.eval(ctx, conc).await?) {
              result = Some(expr.eval(ctx, conc).await?);
              break;
            }
          }
          result.unwrap_or(default.eval(ctx, conc).await?)
        }
        Concurrency::Parallel => {
          let true_cond_index = join_all(list.iter().map(|(cond, _)| cond.eval(ctx, conc)))
            .await
            .into_iter()
            .enumerate()
            .find_map(|(index, cond)| Some(is_truthy_ref(cond.as_ref().ok()?)).map(|_| index));

          if let Some(index) = true_cond_index {
            let (_, value) = list
              .get(index)
              .ok_or(anyhow::anyhow!("no condition found at index {index}"))?;
            value.eval(ctx, conc).await?
          } else {
            default.eval(ctx, conc).await?
          }
        }
      },
      Logic::DefaultTo(value, default) => {
        let result = value.eval(ctx, conc).await?;
        if is_empty(&result) {
          default.eval(ctx, conc).await?
        } else {
          result
        }
      }
      Logic::IsEmpty(expr) => is_empty(&expr.eval(ctx, conc).await?).into(),
      Logic::Not(expr) => (!is_truthy(expr.eval(ctx, conc).await?)).into(),

      Logic::And(list) => {
        let future_iter = list.iter().map(|expr| expr.eval(ctx, conc));

        match *conc {
          Concurrency::Parallel => {
            let mut futures: FuturesUnordered<_> = future_iter.collect();
            let mut output = true;

            while let Some(result) = futures.next().await {
              let result: Result<ConstValue> = result;
              if !is_truthy(result?) {
                output = false;
                break;
              }
            }
            output
          }
          Concurrency::Sequential => {
            let mut output = true;
            for future in future_iter.into_iter() {
              if !is_truthy(future.await?) {
                output = false;
                break;
              }
            }
            output
          }
        }
        .into()
      }
      Logic::If { cond, then, els } => {
        let cond = cond.eval(ctx, conc).await?;
        if is_truthy(cond) {
          then.eval(ctx, conc).await?
        } else {
          els.eval(ctx, conc).await?
        }
      }
    })
  }
}
