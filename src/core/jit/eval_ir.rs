use std::borrow::Cow;
use std::ops::Deref;
use headers::HeaderMap;
use serde_json_borrow::Value;

use crate::core::app_context::AppContext;
use crate::core::has_headers::HasHeaders;
use crate::core::jit::req_ctx::RequestContext;
use crate::core::ir::{Error, GraphQLOperationContext, RelatedFields};
use crate::core::ir::model::CacheKey;
use crate::core::jit::eval_io::eval_io;
use crate::core::jit::ir::IR;
use crate::core::path::{PathGraphql, PathString};

/// An async executor for the IR.
pub struct Eval<'a> {
    #[allow(unused)]
    app_ctx: AppContext,
    req_ctx: RequestContext<'a>,
}

impl GraphQLOperationContext for Eval<'_> {
    fn selection_set(&self, _related_fields: &RelatedFields) -> Option<String> {
        todo!()
    }
}

impl HasHeaders for Eval<'_> {
    fn headers(&self) -> &HeaderMap {
        todo!()
    }
}

impl PathGraphql for Eval<'_> {
    fn path_graphql<T: AsRef<str>>(&self, _path: &[T]) -> Option<String> {
        todo!()
    }
}

impl PathString for Eval<'_> {
    fn path_string<T: AsRef<str>>(&self, _path: &[T]) -> Option<Cow<'_, str>> {
        todo!()
    }
}

impl<'a> Eval<'a> {
    pub fn app_ctx(&self) -> &AppContext {
        &self.app_ctx
    }

    pub fn req_ctx(&self) -> &RequestContext<'a> {
        &self.req_ctx
    }
    pub fn new(app_ctx: AppContext, req_ctx: RequestContext<'a>) -> Self {
        Self { app_ctx, req_ctx }
    }

    #[async_recursion::async_recursion]
    #[allow(clippy::only_used_in_recursion)]
    pub async fn eval(
        &'a self,
        ir: &'a IR,
        value: Option<Value<'a>>,
    ) -> Result<Value<'a>, Error> {
        match ir {
            IR::Path(path) => {
                let value = value.unwrap_or(Value::Null);
                let value = get_path(value, path).unwrap_or(Value::Null);
                Ok(value)
            }
            IR::Dynamic(value) => Ok(Value::from(value)),
            IR::IO(io) => {
                let val = eval_io(io, &self).await?;
                Ok(val)
            }
            IR::Cache(_) => todo!(),
            IR::Protect => todo!(),
            IR::Map(_) => todo!(),
            IR::Pipe(first, second) => {
                let first = self.eval(first, value).await?;
                self.eval(second, Some(first)).await
            }
        }
    }
}

fn get_path<'a, T: AsRef<str>>(value: Value<'a>, path: &'a [T]) -> Option<Value<'a>> {
    let (head, tail) = path.split_first()?;
    let value = match value {
        Value::Object(map) => map
            .into_vec()
            .into_iter()
            .find(|(k, _)| k == head.as_ref())
            .map(|(_, v)| v),
        Value::Array(arr) => {
            let index = head.as_ref().parse::<usize>().ok()?;
            arr.into_iter().nth(index)
        }
        _ => None,
    };

    if tail.is_empty() {
        value
    } else {
        get_path(value?, tail)
    }
}

#[cfg(test)]
mod tests {
    use serde_json_borrow::ObjectAsVec;

    use super::*;

    #[test]
    fn test_resolve_path_obj() {
        let mut obj = ObjectAsVec::default();
        obj.insert("a", Value::Str("b".into()));
        let json = Value::Object(obj);

        let path = vec!["a"];
        let result = get_path(json, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Value::Str("b".into()));
    }

    #[test]
    fn test_resolve_path_arr() {
        let arr = vec![
            Value::Str("a".into()),
            Value::Str("b".into()),
            Value::Str("c".into()),
        ];

        let json = Value::Array(arr);
        let path = vec!["2"];
        let result = get_path(json, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Value::Str("c".into()));
    }

    #[test]
    fn test_resolve_path_obj_and_arr() {
        let mut obj = ObjectAsVec::default();
        obj.insert("a", Value::Str("b".into()));
        let json = Value::Object(obj);

        let arr = vec![Value::Str("a".into()), json, Value::Str("c".into())];

        let json = Value::Array(arr);
        let path = vec!["1", "a"];
        let result = get_path(json, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Value::Str("b".into()));
    }
}
