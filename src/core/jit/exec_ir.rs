use std::borrow::Cow;
use std::ops::Deref;
use serde_json_borrow::Value;

use crate::core::data_loader::DedupeResult;
use crate::core::ir::model::IoId;
use crate::core::ir::Error;
use crate::core::jit::ir::{Context, IR};
use crate::core::runtime::TargetRuntime;

/// An async executor for the IR.
struct Exec<'a> {
    runtime: TargetRuntime,
    args: Option<Value<'a>>,
    store: DedupeResult<IoId, Value<'a>, Error>,
}

impl<'a> Exec<'a> {
    pub fn new(runtime: TargetRuntime) -> Self {
        Self { runtime, args: None, store: DedupeResult::new(true) }
    }

    pub fn execute(&self, ir: IR, value: Option<&'a Value<'a>>) -> Result<Value<'a>, Error> {
        match ir {
            IR::Context(ctx) => {
                match ctx {
                    Context::Value => {
                        Ok(value.cloned().unwrap_or(Value::Null))
                    }
                    Context::Path(path) => {
                        Ok(resolve_path(value, &path).unwrap_or(Value::Null))
                    }
                }
            },
            IR::Dynamic(_) => todo!(),
            IR::IO(_) => todo!(),
            IR::Cache(_) => todo!(),
            IR::Path(_, _) => todo!(),
            IR::Protect(_) => todo!(),
            IR::Map(_) => todo!(),
            IR::Pipe(_, _) => todo!()
        }
    }
}

fn resolve_path<'a, T: AsRef<str>>(mut value: Option< &'a Value<'a>>, path: &[T]) -> Option<Value<'a>> {
    for name in path.iter() {
        match value {
            Some(Value::Object(map)) => {
                value = map.get(name.as_ref());
            }
            Some(Value::Array(list)) => {
                value = list.get(name.as_ref().parse::<usize>().ok()?);
            }
            _ => return None,
        }
    }

    value.cloned()
}

#[cfg(test)]
mod tests {
    use serde_json_borrow::ObjectAsVec;
    use super::*;
    #[test]
    fn test_resolve_path_obj() {
        let mut obj = ObjectAsVec::default();
        obj.insert("a".into(), Value::Str("b".into()));
        let json = Value::Object(obj);

        let path = vec!["a"];
        let result = resolve_path(Some(&json), &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Value::Str("b".into()));
    }

    #[test]
    fn test_resolve_path_arr() {
        let mut arr = vec![];
        arr.push(Value::Str("a".into()));
        arr.push(Value::Str("b".into()));
        arr.push(Value::Str("c".into()));

        let json = Value::Array(arr);
        let path = vec!["2"];
        let result = resolve_path(Some(&json), &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Value::Str("c".into()));
    }

    #[test]
    fn test_resolve_path_obj_and_arr() {
        let mut obj = ObjectAsVec::default();
        obj.insert("a".into(), Value::Str("b".into()));
        let json = Value::Object(obj);

        let mut arr = vec![];
        arr.push(Value::Str("a".into()));
        arr.push(json);
        arr.push(Value::Str("c".into()));

        let json = Value::Array(arr);
        let path = vec!["1","a"];
        let result = resolve_path(Some(&json), &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Value::Str("b".into()));
    }
}