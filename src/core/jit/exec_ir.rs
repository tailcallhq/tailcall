use serde_json_borrow::Value;

use crate::core::ir::Error;
use crate::core::jit::ir::IR;
use crate::core::runtime::TargetRuntime;

/// An async executor for the IR.
pub struct Exec {
    #[allow(unused)]
    runtime: TargetRuntime,
}

impl Exec {
    pub fn new(runtime: TargetRuntime) -> Self {
        Self { runtime }
    }

    #[async_recursion::async_recursion]
    #[allow(clippy::only_used_in_recursion)]
    pub async fn execute<'a>(
        &'a self,
        ir: &'a IR,
        value: Option<&'a Value<'a>>,
    ) -> Result<&'a Value<'a>, Error> {
        match ir {
            IR::Path(path) => {
                let var_name = value.unwrap_or(&Value::Null);
                let value = resolve_path(var_name, path).unwrap_or(&Value::Null);
                Ok(value)
            }
            IR::Dynamic(_) => todo!(),
            IR::IO(_) => todo!(),
            IR::Cache(_) => todo!(),
            IR::Protect(_) => todo!(),
            IR::Map(_) => todo!(),
            IR::Pipe(first, second) => {
                let first = self.execute(first, value).await?;
                self.execute(second, Some(first)).await
            }
        }
    }
}

fn resolve_path<'a, T: AsRef<str>>(value: &'a Value<'a>, path: &'a [T]) -> Option<&'a Value<'a>> {
    let (head, tail) = path.split_first()?;

    match value {
        Value::Object(map) => {
            let value = map.get(head.as_ref());
            if tail.is_empty() {
                value
            } else {
                resolve_path(value?, tail)
            }
        }
        Value::Array(arr) => {
            let index = head.as_ref().parse::<usize>().ok()?;
            let value = arr.get(index);
            if tail.is_empty() {
                value
            } else {
                resolve_path(value?, tail)
            }
        }
        _ => None,
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
        let result = resolve_path(&json, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &Value::Str("b".into()));
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
        let result = resolve_path(&json, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &Value::Str("c".into()));
    }

    #[test]
    fn test_resolve_path_obj_and_arr() {
        let mut obj = ObjectAsVec::default();
        obj.insert("a", Value::Str("b".into()));
        let json = Value::Object(obj);

        let arr = vec![Value::Str("a".into()), json, Value::Str("c".into())];

        let json = Value::Array(arr);
        let path = vec!["1", "a"];
        let result = resolve_path(&json, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &Value::Str("b".into()));
    }
}
