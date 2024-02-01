use core::future::Future;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::ConstValue;

use super::{Concurrent, Eval, EvaluationContext, Expression, ResolverContextLike};
use crate::lambda::has_io::HasIO;

#[derive(Clone, Debug)]
pub enum Logic {
    If {
        cond: Box<Expression>,
        then: Box<Expression>,
        els: Box<Expression>,
    },
    And(Vec<Expression>),
    Or(Vec<Expression>),
    Cond(Vec<(Box<Expression>, Box<Expression>)>),
    DefaultTo(Box<Expression>, Box<Expression>),
    IsEmpty(Box<Expression>),
    Not(Box<Expression>),
}

impl HasIO for Logic {
    fn has_io(&self) -> bool {
        match self {
            Logic::If { cond, then, els } => (cond, then, els).has_io(),
            Logic::And(exprs) => exprs.has_io(),
            Logic::Or(exprs) => exprs.has_io(),
            Logic::Cond(exprs) => exprs.has_io(),
            Logic::DefaultTo(expr1, expr2) => (expr1, expr2).has_io(),
            Logic::IsEmpty(expr) => expr.has_io(),
            Logic::Not(expr) => expr.has_io(),
        }
    }
}

impl Eval for Logic {
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: &'a EvaluationContext<'a, Ctx>,
        conc: &'a Concurrent,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
        Box::pin(async move {
            Ok(match self {
                Logic::Or(list) => {
                    let future_iter = list
                        .iter()
                        .map(|expr| async move { expr.eval(ctx, conc).await });

                    conc.fold(future_iter, false, |acc, val| Ok(acc || is_truthy(&val?)))
                        .await
                        .map(ConstValue::from)?
                }
                Logic::Cond(list) => {
                    for (cond, expr) in list.iter() {
                        if is_truthy(&cond.eval(ctx, conc).await?) {
                            return expr.eval(ctx, conc).await;
                        }
                    }
                    ConstValue::Null
                }
                Logic::DefaultTo(value, default) => {
                    let result = value.eval(ctx, conc).await?;
                    if is_empty(&result) {
                        default.eval(ctx, conc).await?
                    } else {
                        result
                    }
                }
                Logic::IsEmpty(expr) => is_empty(&expr.eval(ctx, conc).await?).into(),
                Logic::Not(expr) => (!is_truthy(&expr.eval(ctx, conc).await?)).into(),

                Logic::And(list) => {
                    let future_iter = list
                        .iter()
                        .map(|expr| async move { expr.eval(ctx, conc).await });

                    conc.fold(future_iter, true, |acc, val| Ok(acc && is_truthy(&val?)))
                        .await
                        .map(ConstValue::from)?
                }
                Logic::If { cond, then, els } => {
                    let cond = cond.eval(ctx, conc).await?;
                    if is_truthy(&cond) {
                        then.eval(ctx, conc).await?
                    } else {
                        els.eval(ctx, conc).await?
                    }
                }
            })
        })
    }
}

/// Check if a value is truthy
///
/// Special cases:
/// 1. An empty string is considered falsy
/// 2. A collection of bytes is truthy, even if the value in those bytes is 0. An empty collection is falsy.
pub fn is_truthy(value: &async_graphql::Value) -> bool {
    use async_graphql::{Number, Value};
    use hyper::body::Bytes;

    match value {
        &Value::Null => false,
        &Value::Enum(_) => true,
        &Value::List(_) => true,
        &Value::Object(_) => true,
        Value::String(s) => !s.is_empty(),
        &Value::Boolean(b) => b,
        Value::Number(n) => n != &Number::from(0),
        Value::Binary(b) => b != &Bytes::default(),
    }
}

fn is_empty(value: &async_graphql::Value) -> bool {
    match value {
        ConstValue::Null => true,
        ConstValue::Number(_) | ConstValue::Boolean(_) | ConstValue::Enum(_) => false,
        ConstValue::Binary(bytes) => bytes.is_empty(),
        ConstValue::List(list) => list.is_empty(),
        ConstValue::Object(obj) => obj.is_empty(),
        ConstValue::String(string) => string.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::{Name, Number, Value};
    use hyper::body::Bytes;
    use indexmap::IndexMap;

    use crate::lambda::is_truthy;

    #[test]
    fn test_is_truthy() {
        assert!(is_truthy(&Value::Enum(Name::new("EXAMPLE"))));
        assert!(is_truthy(&Value::List(vec![])));
        assert!(is_truthy(&Value::Object(IndexMap::default())));
        assert!(is_truthy(&Value::String("Hello".to_string())));
        assert!(is_truthy(&Value::Boolean(true)));
        assert!(is_truthy(&Value::Number(Number::from(1))));
        assert!(is_truthy(&Value::Binary(Bytes::from_static(&[0, 1, 2]))));

        assert!(!is_truthy(&Value::Null));
        assert!(!is_truthy(&Value::String("".to_string())));
        assert!(!is_truthy(&Value::Boolean(false)));
        assert!(!is_truthy(&Value::Number(Number::from(0))));
        assert!(!is_truthy(&Value::Binary(Bytes::default())));
    }
}
