pub use serde_json_borrow::*;

use super::model::{Children, Field, FieldId};
use super::store::{Data, Store};
use crate::core::ir::IoId;

#[allow(unused)]
pub struct Synth {
    operation: Field<Children>,
    store: Store<IoId, OwnedValue>,
}

#[allow(unused)]
impl Synth {
    pub fn new(operation: Field<Children>, store: Store<IoId, OwnedValue>) -> Self {
        Synth { operation, store }
    }

    pub fn synthesize(&self) -> Value<'_> {
        let value = self.store.get(&IoId::new(0));
        self.iter(&self.operation, value)
    }

    pub fn iter<'a>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Data<IoId, OwnedValue>>,
    ) -> Value<'a> {
        match parent {
            Some(value) => {
                match value.data.as_ref().map(|v| v.get_value()) {
                    Some(val) => self.foo(node, Some(val)),
                    _ => {
                        if let Some(key) = value.deferred.get(&(FieldId::new(node.id.0 + 1))) {
                            // TODO: remove node.id.0 + 1 it is just used for the example
                            let value = self.store.get(key);
                            // println!("{:?}", value.map(|v| v.data.as_ref()).flatten());
                            self.iter(&node, value)
                        } else {
                            Value::Null
                        }
                    }
                }
            }
            None => Value::Null,
        }
    }
    fn foo<'a>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Value<'a>>,
    ) -> Value<'a> {
        match parent {
            Some(Value::Object(obj)) => {
                let mut ans = vec![];
                let children = node.children();
                if children.is_empty() {
                    // if it's a leaf node, then push the value
                    let val = obj.iter().find(|(k, _)| node.name.eq(*k)).map(|(_, v)| v);
                    if let Some(val) = val {
                        ans.push((node.name.to_owned(), val.clone()));
                    }
                } else {
                    // if it has children, then pick value from obj and pass it to children.
                    for child in children {
                        let val = obj.iter().find(|(k, _)| child.name.eq(*k)).map(|(_, v)| v);
                        if let Some(val) = val {
                            ans.push((child.name.to_owned(), self.foo(child, Some(val))));
                        }
                    }
                }

                /*for child in node.children() {
                    TODO: pick child from the data and pass it to child
                }*/
                // TODO: in case of resolver.. think think
                Value::Object(ans.into())
            }
            Some(Value::Array(arr)) => {
                let mut ans = vec![];
                for val in arr {
                    ans.push(self.foo(node, Some(val)));
                }
                Value::Object(vec![(node.name.to_owned(), Value::Array(ans))].into())
            }
            Some(val) => val.clone(),
            None => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use insta::assert_snapshot;
    use serde_json_borrow::OwnedValue;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::ir::jit::builder::ExecutionPlanBuilder;
    use crate::core::ir::jit::store::{Data, Store};
    use crate::core::ir::jit::synth::Synth;
    use crate::core::ir::IoId;
    use crate::core::ir::jit::model::FieldId;
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn synth(query: &str) -> String {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        let mut store = Store::new();
        let edoc = ExecutionPlanBuilder::new(blueprint, document).build();

        enum IO {
            Root,
            Post,
            Users,
        }

        let mut deferred = HashMap::new();
        deferred.insert(FieldId::new(IO::Post as usize), IoId::new(IO::Post as u64));

        // Insert Root
        store.insert(
            IoId::new(IO::Root as u64),
            Data {
                data: None,
                deferred,
            },
        );

        let mut deferred = HashMap::new();
        deferred.insert(FieldId::new(IO::Users as usize), IoId::new(IO::Users as u64));
        // Insert /posts
        store.insert(
            IoId::new(IO::Post as u64),
            Data {
                data: Some(
                    OwnedValue::from_str(
                        r#"[{"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": "2"}]"#,
                    )
                        .unwrap(),
                ),
                deferred,
            },
        );

        let deferred = HashMap::new();
        // Insert /user/:id
        store.insert(
            IoId::new(IO::Users as u64),
            Data {
                data: Some(
                    OwnedValue::from_str(
                        r#"{"name": "John Doe", "userId": "1"}"#,
                    ).unwrap()
                ),
                deferred,
            },
        );

        let deferred = HashMap::new();
        store.insert(
            IoId::new(1),
            Data {
                data: Some(
                    OwnedValue::from_str(
                        r#"[{"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": "2"}]"#,
                    ).unwrap()
                ),
                deferred,
            },
        );

        // println!("{:#?}", store);

        // Synthesize the final value
        let children = edoc.into_children();
        println!("{:#?}",children);
        let synth = Synth::new(children.first().unwrap().to_owned(), store);

        synth.synthesize().to_string()
    }

    #[tokio::test]
    async fn test_synth() {
        let actual = synth(
            r#"
                query {
                    posts { title body userId }
                }
            "#,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_users() {
        let actual = synth(
            r#"
                query {
                    users { name address { street } }
                }
            "#,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_post_id() {
        let actual = synth(
            r#"
                query {
                    post(id: 1) { id title }
                }
            "#,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_post_id_to_user() {
        let actual = synth(
            r#"
                query {
                    post(id: 1) { id title user { name } }
                }
            "#,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_all_posts_users() {
        let actual = synth(
            r#"
                query {
                    posts { id title user { name } }
                }
            "#,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_all_posts_users_todos() {
        let actual = synth(
            r#"
                query {
                    posts { title user { name todo { title completed } } }
                }
            "#,
        );

        assert_snapshot!(actual);
    }
}
