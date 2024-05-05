use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use async_graphql::{SelectionField, ServerError, Value};
use async_graphql_value::{ConstValue, Name};
use indexmap::IndexMap;
use jaq_interpret::{Error, Range, ValR2};
use jaq_interpret::error::Type;
use jaq_interpret::results::box_once;
use jaq_syn::path::Opt;
use num_bigint::BigInt;
use reqwest::header::HeaderMap;

use super::{GraphQLOperationContext, ResolverContextLike};
use crate::http::RequestContext;
use crate::json::JsonLike;

// TODO: rename to ResolverContext
#[derive(Clone)]
pub struct EvaluationContext<'a, Ctx: ResolverContextLike<'a>> {
    // Context create for each GraphQL Request
    pub request_ctx: &'a RequestContext,

    // Async GraphQL Context
    // Contains current value and arguments
    graphql_ctx: &'a Ctx,

    // Overridden Value for Async GraphQL Context
    graphql_ctx_value: Option<Arc<Value>>,

    // Overridden Arguments for Async GraphQL Context
    graphql_ctx_args: Option<Arc<Value>>,
}

impl<'a, A: ResolverContextLike<'a>> EvaluationContext<'a, A> {
    pub fn with_value(&self, value: Value) -> EvaluationContext<'a, A> {
        let mut ctx = self.clone();
        ctx.graphql_ctx_value = Some(Arc::new(value));
        ctx
    }

    pub fn with_args(&self, args: Value) -> EvaluationContext<'a, A> {
        let mut ctx = self.clone();
        ctx.graphql_ctx_args = Some(Arc::new(args));
        ctx
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> EvaluationContext<'a, Ctx> {
    pub fn new(req_ctx: &'a RequestContext, graphql_ctx: &'a Ctx) -> EvaluationContext<'a, Ctx> {
        Self {
            request_ctx: req_ctx,
            graphql_ctx,
            graphql_ctx_value: None,
            graphql_ctx_args: None,
        }
    }

    pub fn value(&self) -> Option<&ConstVal> {
        self.graphql_ctx.value().map(|v|v.into())
    }

    pub fn path_arg<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'a, Value>> {
        // TODO: add unit tests for this
        if let Some(args) = self.graphql_ctx_args.as_ref() {
            get_path_value(args.as_ref(), path).map(|a| Cow::Owned(a.clone()))
        } else if path.is_empty() {
            self.graphql_ctx
                .args()
                .map(|a| Cow::Owned(Value::Object(a.clone())))
        } else {
            let arg = self.graphql_ctx.args()?.get(path[0].as_ref())?;
            get_path_value(arg, &path[1..]).map(Cow::Borrowed)
        }
    }

    pub fn path_value<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'a, Value>> {
        // TODO: add unit tests for this
        if let Some(value) = self.graphql_ctx_value.as_ref() {
            get_path_value(value.as_ref(), path).map(|a| Cow::Owned(a.clone()))
        } else {
            get_path_value(self.graphql_ctx.value()?, path).map(Cow::Borrowed)
        }
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.request_ctx.allowed_headers
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        let value = self.headers().get(key)?;

        value.to_str().ok()
    }

    pub fn env_var(&self, key: &str) -> Option<Cow<'_, str>> {
        self.request_ctx.runtime.env.get(key)
    }

    pub fn var(&self, key: &str) -> Option<&str> {
        let vars = &self.request_ctx.server.vars;

        vars.get(key).map(|v| v.as_str())
    }

    pub fn vars(&self) -> &BTreeMap<String, String> {
        &self.request_ctx.server.vars
    }

    pub fn add_error(&self, error: ServerError) {
        self.graphql_ctx.add_error(error)
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> GraphQLOperationContext for EvaluationContext<'a, Ctx> {
    fn selection_set(&self) -> Option<String> {
        let selection_set = self.graphql_ctx.field()?.selection_set();

        format_selection_set(selection_set)
    }
}

fn format_selection_set<'a>(
    selection_set: impl Iterator<Item=SelectionField<'a>>,
) -> Option<String> {
    let set = selection_set
        .map(format_selection_field)
        .collect::<Vec<_>>();

    if set.is_empty() {
        return None;
    }

    Some(format!("{{ {} }}", set.join(" ")))
}

fn format_selection_field(field: SelectionField) -> String {
    let name = field.name();
    let arguments = format_selection_field_arguments(field);
    let selection_set = format_selection_set(field.selection_set());

    if let Some(set) = selection_set {
        format!("{}{} {}", name, arguments, set)
    } else {
        format!("{}{}", name, arguments)
    }
}

fn format_selection_field_arguments(field: SelectionField) -> Cow<'static, str> {
    let name = field.name();
    let arguments = field
        .arguments()
        .map_err(|error| {
            tracing::warn!("Failed to resolve arguments for field {name}, due to error: {error}");

            error
        })
        .unwrap_or_default();

    if arguments.is_empty() {
        return Cow::Borrowed("");
    }

    let args = arguments
        .iter()
        .map(|(name, value)| format!("{}: {}", name, value))
        .collect::<Vec<_>>()
        .join(",");

    Cow::Owned(format!("({})", args))
}

// TODO: this is the same code as src/json/json_like.rs::get_path
pub fn get_path_value<'a, T: AsRef<str>>(input: &'a Value, path: &[T]) -> Option<&'a Value> {
    let mut value = Some(input);
    for name in path {
        match value {
            Some(Value::Object(map)) => {
                value = map.get(name.as_ref());
            }

            Some(Value::List(list)) => {
                value = list.get(name.as_ref().parse::<usize>().ok()?);
            }
            _ => return None,
        }
    }

    value
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstVal(pub ConstValue);

impl<'a> From<&ConstValue> for &'a ConstVal {
    fn from(val: &ConstValue) -> Self {
        unsafe { &*(val as *const ConstValue as *const ConstVal) }
    }

}

impl PartialOrd<Self> for ConstVal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConstVal {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        match (&self.0, &other.0) {
            (Value::Null, Value::Null) => Equal,
            (Value::Boolean(x), Value::Boolean(y)) => x.cmp(y),
            (Value::Number(x), Value::Number(y)) => {
                // Compare numbers based on their types
                match (x, y) {
                    (left, right) if left.is_u64() && right.is_u64() => {
                        left.as_u64().cmp(&right.as_u64())
                    }
                    (left, right) if left.is_i64() && right.is_i64() => {
                        left.as_i64().cmp(&right.as_i64())
                    }
                    _ => Less, // TODO: fixme
                }
            }
            (Value::String(x), Value::String(y)) => x.cmp(y),
            (Value::List(x), Value::List(y)) => {
                // Compare lists lexicographically
                for (a, b) in x.iter().zip(y.iter()) {
                    let cmp = ConstVal(a.clone()).cmp(&ConstVal(b.clone()));
                    if cmp != Equal {
                        return cmp;
                    }
                }
                x.len().cmp(&y.len())
            }
            (Value::Object(x), Value::Object(y)) => {
                // Compare objects based on their keys and values
                let mut l: Vec<_> = x.iter().collect();
                let mut r: Vec<_> = y.iter().collect();
                l.sort_by_key(|(k, _v)| *k);
                r.sort_by_key(|(k, _v)| *k);
                let kl = l.iter().map(|(k, _v)| k);
                let kr = r.iter().map(|(k, _v)| k);
                let vl = l.iter().map(|(_k, v)| ConstVal::from((*v).clone()));
                let vr = r.iter().map(|(_k, v)|  ConstVal::from((*v).clone()));
                kl.cmp(kr).then_with(|| vl.cmp(vr))
            }
            // Nulls are smaller than anything else
            (Value::Null, _) => Less,
            (_, Value::Null) => Greater,
            // Booleans are smaller than anything else, except for nulls
            (Value::Boolean(_), _) => Less,
            (_, Value::Boolean(_)) => Greater,
            // Numbers are smaller than anything else, except for nulls and booleans
            (Value::Number(_), _) => Less,
            (_, Value::Number(_)) => Greater,
            // etc.
            (Value::String(_), _) => Less,
            (_, Value::String(_)) => Greater,
            (Value::List(_), _) => Less,
            (_, Value::List(_)) => Greater,
            (Value::Object(_), _) => Less,
            (_, Value::Object(_)) => Greater,
            _ => Less, // TODO: fixme
        }
    }
}

impl Display for ConstVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for ConstVal {
    fn from(val: bool) -> Self {
        Self(ConstValue::from(val))
    }
}

impl From<ConstValue> for ConstVal {
    fn from(val: ConstValue) -> Self {
        Self(val)
    }
}

impl From<&ConstVal> for ConstValue {
    fn from(val: &ConstVal) -> Self {
        val.0.clone()
    }
}

impl From<ConstVal> for Value {
    fn from(val: ConstVal) -> Self {
        val.0.into()
    }
}

impl From<isize> for ConstVal {
    fn from(value: isize) -> Self {
        Self(ConstValue::from(value))
    }
}

impl From<String> for ConstVal {
    fn from(value: String) -> Self {
        Self(ConstValue::from(value))
    }
}

impl FromIterator<Self> for ConstVal {
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        Self(ConstValue::List(iter.into_iter().map(|v|v.0).collect()))
    }
}

impl core::ops::Sub for ConstVal {
    type Output = ValR2<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        let val = match (self.0, rhs.0) {
            (ConstValue::Number(a), ConstValue::Number(b)) => match (a,b) {
                (a,b) => {
                    let a = BigInt::from_str(a.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let b = BigInt::from_str(b.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let c = a-b;
                    let val = c.to_string().parse::<i64>().map_err(|e| Error::<ConstVal>::str(Error::<ConstVal>::str(e.to_string()))).map(|v| ConstValue::from(v))?;
                    Ok(val)
                }
            },
            (ConstValue::List(a), ConstValue::List(b)) => Ok(ConstValue::List(a.into_iter().filter(|v| !b.contains(v)).collect())),// TODO use set to improve performance
            (a, _) => Err(Error::<ConstVal>::str("fixme")),
        };
        Ok(val.map(|v| v.into()).map_err(|_| Error::<ConstVal>::str("fixme"))?)
    }
}

impl core::ops::Mul for ConstVal {
    type Output = ValR2<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        let val = match (self.0, rhs.0) {
            (ConstValue::Number(a), ConstValue::Number(b)) => match (a,b) {
                (a,b) => {
                    let a = BigInt::from_str(a.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let b = BigInt::from_str(b.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let c = a*b;
                    let val = c.to_string().parse::<i64>().map_err(|e| Error::<ConstVal>::str(e.to_string())).map(|v| ConstValue::from(v))?;
                    Ok(val)
                }
            },
            (ConstValue::String(a), ConstValue::Number(b)) => Ok(ConstValue::String(a.repeat(b.as_i64().unwrap() as usize))),
            (ConstValue::Number(a), ConstValue::String(b)) => Ok(ConstValue::String(b.repeat(a.as_i64().unwrap() as usize))),
            (ConstValue::List(a), ConstValue::Number(b)) => Ok(ConstValue::List(a.into_iter().flat_map(|v| std::iter::repeat(v).take(b.as_i64().unwrap() as usize)).collect())),
            (ConstValue::Number(b), ConstValue::List(a)) => Ok(ConstValue::List(a.into_iter().flat_map(|v| std::iter::repeat(v).take(b.as_i64().unwrap() as usize)).collect())),
            (a, _) => Err(Error::<ConstVal>::str("fixme")),
        };
        Ok(val.map(|v| v.into()).map_err(|_|Error::<ConstVal>::str("fixme"))?)
    }
}

impl core::ops::Div for ConstVal {
    type Output = ValR2<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        let val = match (self.0, rhs.0) {
            (ConstValue::Number(a), ConstValue::Number(b)) => match (a,b) {
                (a,b) => {
                    let a = BigInt::from_str(a.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let b = BigInt::from_str(b.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let c = a/b;
                    let val = c.to_string().parse::<i64>().map_err(|e| Error::<ConstVal>::str(e.to_string())).map(|v| ConstValue::from(v))?;
                    Ok(val)
                }
            },
            (ConstValue::List(a), ConstValue::Number(b)) => Ok(ConstValue::List(a.into_iter().cycle().take(b.as_i64().unwrap() as usize).collect())),
            (a, _) => Err(Error::<ConstVal>::str("fixme")),
        };
        val.map(|v| v.into()).map_err(|_| Error::<ConstVal>::str("fixme"))
    }
}

impl core::ops::Rem for ConstVal {
    type Output = ValR2<Self>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self.0, rhs.0) {
            (ConstValue::Number(a), ConstValue::Number(b)) => match (a,b) {
                (a,b) => {
                    let a = BigInt::from_str(a.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let b = BigInt::from_str(b.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let c = a%b;
                    let val = c.to_string().parse::<i64>().map_err(|e| e.to_string()).map(|v| ConstValue::from(v)).map_err(|e| Error::<ConstVal>::str(e))?;
                    Ok(val)
                }
            },
            (a, _) => Err(Error::<ConstVal>::str("fixme")),
        }.map(|v| v.into()).map_err(|_| Error::<ConstVal>::str("fixme"))
    }
}

impl core::ops::Neg for ConstVal {
    type Output = ValR2<Self>;

    fn neg(self) -> Self::Output {
        let val = match self.0 {
            ConstValue::Number(a) => {
                let a = BigInt::from_str(a.as_str()).map_err(|e| e.to_string()).map_err(|e| Error::<ConstVal>::str(e))?;
                let c = -a;
                let val = c.to_string().parse::<i64>().map_err(|e| e.to_string()).map(|v| ConstValue::from(v)).map_err(|e|Error::<ConstVal>::str(e))?;
                Ok(val)
            }
            a => Err(Error::<ConstVal>::str("fixme")),
        };
        Ok(val.map(|v| v.into()).map_err(|_| Error::<ConstVal>::str("fixme"))?)
    }
}

impl core::ops::Add for ConstVal {
    type Output = ValR2<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        let val = match (self.0, rhs.0) {
            (ConstValue::Number(a), ConstValue::Number(b)) => match (a,b) {
                (a,b) => {
                    let a = BigInt::from_str(a.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let b = BigInt::from_str(b.as_str()).map_err(|e| Error::<ConstVal>::str(e.to_string()))?;
                    let c = a+b;
                    let val = c.to_string().parse::<i64>().map_err(|e| e.to_string()).map(|v| ConstValue::from(v)).map_err(|_|Error::<ConstVal>::str("fixme"))?;
                    Ok(val)
                }
            },
            (ConstValue::String(a), ConstValue::String(b)) => Ok(ConstValue::String(a + &b)),
            (ConstValue::List(a), ConstValue::List(b)) => Ok(ConstValue::List(a.into_iter().chain(b).collect())),
            (ConstValue::Object(a), ConstValue::Object(b)) => Ok(ConstValue::Object(a.into_iter().chain(b).collect())),
            (a, _) => Err(Error::<ConstVal>::str("fixme")),
        };
        Ok(val.map(|v| v.into()).map_err(|_|Error::<ConstVal>::str("fixme"))?)
    }
}

impl jaq_interpret::ValT for ConstVal {
    fn from_num(n: String) -> ValR2<Self> {
        let n = n.parse::<i64>().map_err(|e| Error::<ConstVal>::str("fixme"))?;
        let val = Self::from(ConstValue::from(n));
        Ok(val)
    }

    fn from_map<I: IntoIterator<Item=(Self, Self)>>(iter: I) -> ValR2<Self> {
        let map = IndexMap::from_iter(iter.into_iter().map(|(k, v)| (Name::new(k.as_str().map(|v| v.to_string()).unwrap_or(k.to_string())), v.0)));
        Ok(Self::from(ConstValue::from(map)))
    }

    fn values(self) -> Box<dyn Iterator<Item=ValR2<Self>>> {
       /* let val = match self.val {
            ConstValue::List(a) => Box::new(a.into_iter().map(Ok)),
            ConstValue::Object(o) => Box::new(o.into_iter().map(|(k, v)|  v.into())),
            _ => box_once(Err(Error::<ConstVal>::Type(self, Type::Iter))),
        };
        */
        todo!()
    }

    fn index(self, index: &Self) -> ValR2<Self> {
        let val = match (self.0, &index.0) {
            (ConstValue::List(v), ConstValue::Number(num)) => {
                let index = num.as_i64().ok_or_else(|| Error::Type(index.clone(), Type::Int))?;
                Ok(v.get(index as usize).cloned().ok_or(Error::<ConstVal>::str("fixme"))?)
            }
            (ConstValue::Object(map), ConstValue::String(s)) => {
                Ok(map.get(&Name::new(s)).cloned().unwrap_or(ConstValue::Null))
            }
            (s @ (ConstValue::List(_) | ConstValue::Object(_)), _) => Err(Error::<ConstVal>::str("fixme")),
            (s, _) => Err(Error::<ConstVal>::str("fixme")),
        };
        Ok(val.map(|v| v.into()).map_err(|_|Error::<ConstVal>::str("fixme"))?)
    }

    fn range(self, range: Range<&Self>) -> ValR2<Self> {
        let val = match self.0 {
            ConstValue::List(v) => {
                let start = range.start.unwrap().0.as_i64_ok().unwrap();
                let end = range.end.unwrap().0.as_i64_ok().unwrap();
                let start = start as usize;
                let end = end as usize;
                Ok(ConstValue::List(v[start..end].to_vec()))
            }
            ConstValue::String(s) => {
                let start = range.start.unwrap().0.as_i64_ok().ok().ok_or_else(|| Error::Type(range.start.clone(), Type::Int)).unwrap();
                let end = range.end.unwrap().0.as_i64_ok().ok().ok_or_else(|| Error::Type(range.end.clone(), Type::Int)).unwrap();
                let start = start as usize;
                let end = end as usize;
                Ok(ConstValue::String(s[start..end].to_string()))
            }
            s => Err(Error::Type(s, Type::Iter)),
        };
        Ok(val.map(|v| v.into()).map_err(|_| Error::<ConstVal>::str("fixme"))?)
    }

    fn map_values<I: Iterator<Item=ValR2<Self>>>(self, opt: Opt, f: impl Fn(Self) -> I) -> ValR2<Self> {
        let val = match self.0 {
            ConstValue::List(v) => Ok(ConstValue::List(v.into_iter().flat_map(|v| f(v.into())).map(|v|v.unwrap().0).collect())),
            ConstValue::Object(o) => Ok(ConstValue::Object(IndexMap::from_iter(o.into_iter().map(|(k, v)| (k, f(ConstVal(v)).next().unwrap().unwrap().0 // FIXME
            )).into_iter()))),
            s => opt.fail(s, |v| Error::Type(v, Type::Iter)),
        };
        Ok(val.map(|v| v.into()).map_err(|_| Error::<ConstVal>::str("fixme"))?)
    }

    fn map_index<I: Iterator<Item=ValR2<Self>>>(self, index: &Self, opt: Opt, f: impl Fn(Self) -> I) -> ValR2<Self> {
        todo!()
    }

    fn map_range<I: Iterator<Item=ValR2<Self>>>(self, range: Range<&Self>, opt: Opt, f: impl Fn(Self) -> I) -> ValR2<Self> {
        todo!()
    }

    fn as_bool(&self) -> bool {
        todo!()
    }

    fn as_str(&self) -> Option<&str> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Value;
    use serde_json::json;

    use crate::lambda::evaluation_context::get_path_value;

    #[test]
    fn test_path_value() {
        let json = json!(
        {
            "a": {
                "b": {
                    "c": "d"
                }
            }
        });

        let async_value = Value::from_json(json).unwrap();

        let path = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = get_path_value(&async_value, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &Value::String("d".to_string()));
    }

    #[test]
    fn test_path_not_found() {
        let json = json!(
        {
            "a": {
                "b": "c"
            }
        });

        let async_value = Value::from_json(json).unwrap();

        let path = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = get_path_value(&async_value, &path);
        assert!(result.is_none());
    }

    #[test]
    fn test_numeric_path() {
        let json = json!(
        {
            "a": [{
                "b": "c"
            }]
        });

        let async_value = Value::from_json(json).unwrap();

        let path = vec!["a".to_string(), "0".to_string(), "b".to_string()];
        let result = get_path_value(&async_value, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &Value::String("c".to_string()));
    }
}
