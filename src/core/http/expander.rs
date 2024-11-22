use anyhow::Ok;
use serde_json::Value;

use crate::core::blueprint::DynamicValue;

pub struct Expander;

impl Expander {
    // Takes ownership of the request body and returns the expanded Value.
    pub fn expand(
        dynamic_value: &DynamicValue<serde_json::Value>,
        batch_size: usize,
    ) -> anyhow::Result<DynamicValue<serde_json::Value>> {
        if batch_size > 0 {
            let str_value = dynamic_value.to_string()?;
            let value: serde_json::Value = serde_json::from_str(&str_value)?;
            Ok(DynamicValue::Value(Self::expand_inner(value, batch_size)))
        } else {
            Ok(dynamic_value.to_owned())
        }
    }

    fn expand_inner(value: Value, batch_size: usize) -> Value {
        match value {
            Value::Object(map) => {
                let expanded_map = map
                    .into_iter()
                    .map(|(k, v)| (k, Self::expand_inner(v, batch_size)))
                    .collect();
                Value::Object(expanded_map)
            }
            Value::Array(list) => {
                let expanded_list: Vec<Value> = list
                    .into_iter()
                    .map(|v| Self::expand_inner(v, batch_size))
                    .collect();

                let mut final_ans = Vec::with_capacity(expanded_list.len());

                for index in 0..batch_size {
                    let expanded_batch: Vec<Value> = expanded_list
                        .iter()
                        .cloned()
                        .map(|v| Self::update_mustache_expr(v, index))
                        .collect();
                    final_ans.extend(expanded_batch);
                }
                Value::Array(final_ans)
            }
            other => other, // Return as is for other variants.
        }
    }

    fn update_mustache_expr(value: Value, index: usize) -> Value {
        match value {
            Value::Object(map) => {
                let updated_map = map
                    .into_iter()
                    .map(|(k, v)| (k, Self::update_mustache_expr(v, index)))
                    .collect();
                Value::Object(updated_map)
            }
            Value::Array(list) => {
                let updated_list = list
                    .into_iter()
                    .map(|v| Self::update_mustache_expr(v, index))
                    .collect();
                Value::Array(updated_list)
            }
            Value::String(s) => {
                let updated_string = s
                    .replace("{{.value.", &format!("{{{{.value.{}.", index))
                    .replace("{{value.", &format!("{{{{value.{}.", index))
                    .replace("{{.headers.", &format!("{{{{.headers.{}.", index))
                    .replace("{{headers.", &format!("{{{{headers.{}.", index))
                    .replace("{{.vars.", &format!("{{{{.vars.{}.", index))
                    .replace("{{vars.", &format!("{{{{vars.{}.", index))
                    .replace("{{.env.", &format!("{{{{.env.{}.", index))
                    .replace("{{env.", &format!("{{{{env.{}.", index))
                    .replace("{{.args.", &format!("{{{{.args.{}.", index))
                    .replace("{{args.", &format!("{{{{args.{}.", index));
                Value::String(updated_string)
            }
            other => other, // Return as is for other variants.
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_value_expander() {
        let supported_ext = ["value", "headers", "vars", "env", "args"];
        for ext in supported_ext {
            // Test Option 1
            let input1 = json!({
                "a": { "b": { "c": { "d": [format!("{{{{.{}.userId}}}}", ext)] } } }
            });

            let actual = Expander::expand_inner(input1, 2);
            let expected = json!({
                "a": { 
                    "b": { 
                        "c": { 
                            "d": [
                                format!("{{{{.{}.0.userId}}}}", ext), 
                                format!("{{{{.{}.1.userId}}}}", ext)
                            ] 
                        } 
                    } 
                }
            });
            assert_eq!(actual, expected);

            // Test Option 2
            let input2 = json!([
                {
                    "userId": format!("{{{{.{}.id}}}}", ext),
                    "title": format!("{{{{.{}.name}}}}", ext),
                    "content": "Hello World"
                }
            ]);

            let actual = Expander::expand_inner(input2, 2);
            let expected = json!([
                {
                    "userId": format!("{{{{.{}.0.id}}}}", ext),
                    "title": format!("{{{{.{}.0.name}}}}", ext),
                    "content": "Hello World"
                },
                {
                    "userId": format!("{{{{.{}.1.id}}}}", ext),
                    "title": format!("{{{{.{}.1.name}}}}", ext),
                    "content": "Hello World"
                }
            ]);
            assert_eq!(actual, expected);

            // Test Option 3
            let input3 = json!([
                {
                    "metadata": "xyz",
                    "items": format!("{{{{.{}.userId}}}}", ext)
                }
            ]);

            let actual = Expander::expand_inner(input3, 2);
            let expected = json!([
                {
                    "metadata": "xyz",
                    "items": format!("{{{{.{}.0.userId}}}}", ext)
                },
                {
                    "metadata": "xyz",
                    "items": format!("{{{{.{}.1.userId}}}}", ext)
                }
            ]);
            assert_eq!(actual, expected);

            // Test Option 4
            let input4 = json!({
                "metadata": "xyz",
                "items": [
                    {
                        "key": "id",
                        "value": format!("{{{{.{}.userId}}}}", ext)
                    }
                ]
            });

            let actual = Expander::expand_inner(input4, 2);
            let expected = json!({
                "metadata": "xyz",
                "items": [
                    {
                        "key": "id",
                        "value": format!("{{{{.{}.0.userId}}}}", ext)
                    },
                    {
                        "key": "id",
                        "value": format!("{{{{.{}.1.userId}}}}", ext)
                    }
                ]
            });
            assert_eq!(actual, expected);
        }
    }
}
