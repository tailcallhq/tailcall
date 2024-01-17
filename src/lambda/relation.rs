use std::cmp::Ordering;
use std::collections::HashSet;

use async_graphql_value::ConstValue;
use futures_util::future::join_all;

use super::{
  eval_list_expressions, get_path_for_const_value_owned, get_path_for_const_value_ref, set_operation, Concurrency,
  Eval, EvaluationContext, EvaluationError, Expression, ResolverContextLike,
};
use crate::helpers::value::{compare, is_list_comparable, HashableConstValue};

#[derive(Clone, Debug)]
pub enum Relation {
  Intersection(Vec<Expression>),
  Difference(Vec<Expression>, Vec<Expression>),
  Equals(Box<Expression>, Box<Expression>),
  Gt(Box<Expression>, Box<Expression>),
  Gte(Box<Expression>, Box<Expression>),
  Lt(Box<Expression>, Box<Expression>),
  Lte(Box<Expression>, Box<Expression>),
  Max(Vec<Expression>),
  Min(Vec<Expression>),
  PathEq(Box<Expression>, Vec<String>, Box<Expression>),
  PropEq(Box<Expression>, String, Box<Expression>),
  SortPath(Box<Expression>, Vec<String>),
  SymmetricDifference(Vec<Expression>, Vec<Expression>),
  Union(Vec<Expression>, Vec<Expression>),
}

impl Eval for Relation {
  async fn async_eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrency,
  ) -> anyhow::Result<async_graphql::Value> {
    Ok(match self {
      Relation::Intersection(exprs) => {
        let results = join_all(exprs.iter().map(|expr| expr.eval(ctx, conc))).await;

        let mut results_iter = results.into_iter();

        let set: HashSet<_> = match results_iter.next() {
          Some(first) => match first? {
            ConstValue::List(list) => list.into_iter().map(HashableConstValue).collect(),
            _ => Err(EvaluationError::IntersectionException("element is not a list".into()))?,
          },
          None => Err(EvaluationError::IntersectionException("element is not a list".into()))?,
        };

        let final_set = results_iter.try_fold(set, |mut acc, result| match result? {
          ConstValue::List(list) => {
            let set: HashSet<_> = list.into_iter().map(HashableConstValue).collect();
            acc = acc.intersection(&set).cloned().collect();
            Ok::<_, anyhow::Error>(acc)
          }
          _ => Err(EvaluationError::IntersectionException("element is not a list".into()))?,
        })?;

        final_set
          .into_iter()
          .map(|HashableConstValue(const_value)| const_value)
          .collect()
      }
      Relation::Difference(lhs, rhs) => {
        set_operation(ctx, conc, lhs, rhs, |lhs, rhs| {
          lhs
            .difference(&rhs)
            .cloned()
            .map(|HashableConstValue(const_value)| const_value)
            .collect()
        })
        .await?
      }
      Relation::Equals(lhs, rhs) => (lhs.eval(ctx, conc).await? == rhs.eval(ctx, conc).await?).into(),
      Relation::Gt(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        (compare(&lhs, &rhs) == Some(Ordering::Greater)).into()
      }
      Relation::Gte(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        matches!(compare(&lhs, &rhs), Some(Ordering::Greater) | Some(Ordering::Equal)).into()
      }
      Relation::Lt(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        (compare(&lhs, &rhs) == Some(Ordering::Less)).into()
      }
      Relation::Lte(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        matches!(compare(&lhs, &rhs), Some(Ordering::Less) | Some(Ordering::Equal)).into()
      }
      Relation::Max(exprs) => {
        let mut results: Vec<_> = eval_list_expressions(ctx, conc, exprs).await?;

        let last = results.pop().ok_or(EvaluationError::OperationFailed(
          "`max` cannot be called on empty list".into(),
        ))?;

        results.into_iter().try_fold(last, |mut largest, current| {
          let ord = compare(&largest, &current);
          largest = match ord {
            Some(Ordering::Greater | Ordering::Equal) => largest,
            Some(Ordering::Less) => current,
            _ => Err(anyhow::anyhow!(
              "`max` cannot be calculated for types that cannot be compared"
            ))?,
          };
          Ok::<_, anyhow::Error>(largest)
        })?
      }
      Relation::Min(exprs) => {
        let mut results: Vec<_> = eval_list_expressions(ctx, conc, exprs).await?;

        let last = results.pop().ok_or(EvaluationError::OperationFailed(
          "`min` cannot be called on empty list".into(),
        ))?;

        results.into_iter().try_fold(last, |mut largest, current| {
          let ord = compare(&largest, &current);
          largest = match ord {
            Some(Ordering::Less | Ordering::Equal) => largest,
            Some(Ordering::Greater) => current,
            _ => Err(anyhow::anyhow!(
              "`min` cannot be calculated for types that cannot be compared"
            ))?,
          };
          Ok::<_, anyhow::Error>(largest)
        })?
      }
      Relation::PathEq(lhs, path, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let lhs = get_path_for_const_value_owned(path, lhs).ok_or(anyhow::anyhow!("Could not find path: {path:?}"))?;

        let rhs = rhs.eval(ctx, conc).await?;
        let rhs = get_path_for_const_value_owned(path, rhs).ok_or(anyhow::anyhow!("Could not find path: {path:?}"))?;

        (lhs == rhs).into()
      }
      Relation::PropEq(lhs, prop, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let lhs =
          get_path_for_const_value_owned(&[prop], lhs).ok_or(anyhow::anyhow!("Could not find path: {prop:?}"))?;

        let rhs = rhs.eval(ctx, conc).await?;
        let rhs =
          get_path_for_const_value_owned(&[prop], rhs).ok_or(anyhow::anyhow!("Could not find path: {prop:?}"))?;

        (lhs == rhs).into()
      }
      Relation::SortPath(expr, path) => {
        let value = expr.eval(ctx, conc).await?;
        let values = match value {
          ConstValue::List(list) => list,
          _ => Err(EvaluationError::OperationFailed(
            "`sortPath` can only be applied to expressions that return list".into(),
          ))?,
        };

        let is_comparable = is_list_comparable(&values);
        let mut values: Vec<_> = values.into_iter().enumerate().collect();

        if !is_comparable {
          Err(anyhow::anyhow!("sortPath requires a list of comparable types"))?
        }

        let value_paths: Vec<_> = values
          .iter()
          .filter_map(|(_, val)| get_path_for_const_value_ref(path, val))
          .cloned()
          .collect();

        if values.len() != value_paths.len() {
          Err(anyhow::anyhow!(
            "path is not valid for all the element in the list: {value_paths:?}"
          ))?
        }

        values.sort_by(|(index1, _), (index2, _)| compare(&value_paths[*index1], &value_paths[*index2]).unwrap());

        values.into_iter().map(|(_, val)| val).collect::<Vec<_>>().into()
      }
      Relation::SymmetricDifference(lhs, rhs) => {
        set_operation(ctx, conc, lhs, rhs, |lhs, rhs| {
          lhs
            .symmetric_difference(&rhs)
            .cloned()
            .map(|HashableConstValue(const_value)| const_value)
            .collect()
        })
        .await?
      }
      Relation::Union(lhs, rhs) => {
        set_operation(ctx, conc, lhs, rhs, |lhs, rhs| {
          lhs
            .union(&rhs)
            .cloned()
            .map(|HashableConstValue(const_value)| const_value)
            .collect()
        })
        .await?
      }
    })
  }
}
