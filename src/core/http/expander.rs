use std::marker::PhantomData;

use anyhow::Ok;
use serde_json::Value;

use crate::core::blueprint::DynamicValue;
use crate::core::mustache::Segment;
use crate::core::Mustache;

const PREFIXES: [&str; 5] = ["value", "headers", "vars", "env", "args"];

/// `Expander` processes `DynamicValue<A>` to expand list types based on `batch_size`,
/// incorporating list indices into mustache expressions.
pub struct Expander<A>(PhantomData<A>);

impl Expander<DynamicValue<async_graphql_value::ConstValue>> {
    pub fn expand(
        dynamic_value: DynamicValue<async_graphql_value::ConstValue>,
        batch_size: usize,
    ) -> anyhow::Result<DynamicValue<async_graphql_value::ConstValue>> {
        if batch_size > 0 {
            Ok(Self::expand_inner(dynamic_value, batch_size))
        } else {
            Ok(dynamic_value.to_owned())
        }
    }

    fn expand_inner(
        value: DynamicValue<async_graphql_value::ConstValue>,
        batch_size: usize,
    ) -> DynamicValue<async_graphql_value::ConstValue> {
        match value {
            DynamicValue::Object(obj) => {
                let expanded_map = obj
                    .into_iter()
                    .map(|(k, v)| (k, Self::expand_inner(v, batch_size)))
                    .collect();
                DynamicValue::Object(expanded_map)
            }
            DynamicValue::Array(arr) => {
                let expanded_list = arr
                    .into_iter()
                    .map(|v| Self::expand_inner(v, batch_size))
                    .collect::<Vec<_>>();

                // copy the list `batch_size` times with replacing the expression with the
                // index.
                let mut ans = Vec::with_capacity(expanded_list.len());

                for index in 0..batch_size {
                    let expanded_batch: Vec<DynamicValue<async_graphql_value::ConstValue>> =
                        expanded_list
                            .iter()
                            .cloned()
                            .map(|v| Self::update_mustache_expr(v, index))
                            .collect();
                    ans.extend(expanded_batch);
                }

                DynamicValue::Array(ans)
            }
            other => other,
        }
    }

    fn update_mustache_expr(
        value: DynamicValue<async_graphql_value::ConstValue>,
        index: usize,
    ) -> DynamicValue<async_graphql_value::ConstValue> {
        match value {
            DynamicValue::Object(map) => {
                let updated_map = map
                    .into_iter()
                    .map(|(k, v)| (k, Self::update_mustache_expr(v, index)))
                    .collect();
                DynamicValue::Object(updated_map)
            }
            DynamicValue::Mustache(mut template) => {
                template.segments_mut().iter_mut().for_each(|segment| {
                    if let Segment::Expression(parts) = segment {
                        let mut modified_pars = Vec::with_capacity(parts.len() + 1);
                        for part in parts.iter() {
                            if PREFIXES.contains(&part.as_str()) {
                                modified_pars.push(part.to_string());
                                modified_pars.push(index.to_string());
                            } else {
                                modified_pars.push(part.to_string());
                            }
                        }
                        *parts = modified_pars;
                    }
                });
                DynamicValue::Mustache(template)
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
    fn test_with_dynamic_value() {
        let expander = |input: serde_json::Value, sz: usize| {
            let dyn_value =
                DynamicValue::<async_graphql_value::ConstValue>::try_from(&input).unwrap();

            Expander::<DynamicValue<async_graphql_value::ConstValue>>::expand(dyn_value, sz)
                .unwrap()
        };

        for ext in PREFIXES {
            // Test Option -1
            let input1 = json!({
                "a": [{"d": { "b": { "c": { "d": [format!("{{{{.{}.userId}}}}", ext)] } } }}]
            });

            let actual = expander(input1, 2);

            let expected = DynamicValue::try_from(&json!({
                "a": [{
                    "d": {
                        "b": {
                            "c" : {
                                "d": [
                                    format!("{{{{{}.0.userId}}}}", ext),
                                    format!("{{{{{}.1.userId}}}}", ext)
                                ]
                            }
                        }
                    }
                },{
                    "d": {
                        "b": {
                            "c" : {
                                "d": [
                                    format!("{{{{{}.0.userId}}}}", ext),
                                    format!("{{{{{}.1.userId}}}}", ext)
                                ]
                            }
                        }
                    }
                }]
            }))
            .unwrap();
            assert_eq!(actual, expected);

            // Test Option -1
            let input1 = json!({
                "a": [[{ "b": { "c": { "d": [format!("{{{{.{}.userId}}}}", ext)] } } }]]
            });

            let actual = expander(input1, 2);

            let expected = DynamicValue::try_from(&json!({
                "a": [[{
                    "b": {
                        "c": {
                            "d": [
                                format!("{{{{{}.0.userId}}}}", ext),
                                format!("{{{{{}.1.userId}}}}", ext)
                            ]
                        }
                    }
                }, {
                    "b": {
                        "c": {
                            "d": [
                                format!("{{{{{}.0.userId}}}}", ext),
                                format!("{{{{{}.1.userId}}}}", ext)
                            ]
                        }
                    }
                }],[{
                    "b": {
                        "c": {
                            "d": [
                                format!("{{{{{}.0.userId}}}}", ext),
                                format!("{{{{{}.1.userId}}}}", ext)
                            ]
                        }
                    }
                }, {
                    "b": {
                        "c": {
                            "d": [
                                format!("{{{{{}.0.userId}}}}", ext),
                                format!("{{{{{}.1.userId}}}}", ext)
                            ]
                        }
                    }
                }]]
            }))
            .unwrap();

            assert_eq!(actual, expected);

            // Test Option 0
            let input1 = json!({
                "a": [{ "b": { "c": { "d": [format!("{{{{.{}.userId}}}}", ext)] } } }]
            });

            let actual = expander(input1, 2);

            let expected = DynamicValue::try_from(&json!({
                "a": [{
                    "b": {
                        "c": {
                            "d": [
                                format!("{{{{{}.0.userId}}}}", ext),
                                format!("{{{{{}.1.userId}}}}", ext)
                            ]
                        }
                    }
                }, {
                    "b": {
                        "c": {
                            "d": [
                                format!("{{{{{}.0.userId}}}}", ext),
                                format!("{{{{{}.1.userId}}}}", ext)
                            ]
                        }
                    }
                }]
            }))
            .unwrap();

            assert_eq!(actual, expected);

            // Test Option 1
            let input1 = json!({
                "a": { "b": { "c": { "d": [format!("{{{{.{}.userId}}}}", ext)] } } }
            });

            let actual = expander(input1, 2);

            let expected = DynamicValue::try_from(&json!({
                "a": {
                    "b": {
                        "c": {
                            "d": [
                                format!("{{{{{}.0.userId}}}}", ext),
                                format!("{{{{{}.1.userId}}}}", ext)
                            ]
                        }
                    }
                }
            }))
            .unwrap();
            assert_eq!(actual, expected);

            // Test Option 2
            let input2 = json!([
                {
                    "userId": format!("{{{{.{}.id}}}}", ext),
                    "title": format!("{{{{.{}.name}}}}", ext),
                    "content": "Hello World"
                }
            ]);

            let actual = expander(input2, 2);
            let expected = DynamicValue::try_from(&json!([
                {
                    "userId": format!("{{{{{}.0.id}}}}", ext),
                    "title": format!("{{{{{}.0.name}}}}", ext),
                    "content": "Hello World"
                },
                {
                    "userId": format!("{{{{{}.1.id}}}}", ext),
                    "title": format!("{{{{{}.1.name}}}}", ext),
                    "content": "Hello World"
                }
            ]))
            .unwrap();
            assert_eq!(actual, expected);

            // Test Option 3
            let input3 = json!([
                {
                    "metadata": "xyz",
                    "items": format!("{{{{.{}.userId}}}}", ext)
                }
            ]);

            let actual = expander(input3, 2);
            let expected = DynamicValue::try_from(&json!([
                {
                    "metadata": "xyz",
                    "items": format!("{{{{{}.0.userId}}}}", ext)
                },
                {
                    "metadata": "xyz",
                    "items": format!("{{{{{}.1.userId}}}}", ext)
                }
            ]))
            .unwrap();
            assert_eq!(actual, expected);

            // Test Option 4
            let input4 = json!({
                "metadata": "xyz",
                "items": [
                    {
                        "key": "id",
                        "value": format!("{{{{{}.userId}}}}", ext)
                    }
                ]
            });

            let actual = expander(input4, 2);
            let expected = DynamicValue::try_from(&json!({
                "metadata": "xyz",
                "items": [
                    {
                        "key": "id",
                        "value": format!("{{{{{}.0.userId}}}}", ext)
                    },
                    {
                        "key": "id",
                        "value": format!("{{{{{}.1.userId}}}}", ext)
                    }
                ]
            }))
            .unwrap();
            assert_eq!(actual, expected);
        }
    }
}
