pub use serde_json_borrow::*;

use super::model::{Children, Field};
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

    fn validate(type_of: &crate::core::blueprint::Type, value: &Value) -> bool {
        type_of.is_list() == value.is_array()
    }

    pub fn iter<'a>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Data<IoId, OwnedValue>>,
    ) -> Value<'a> {
        match parent {
            Some(parent) => match parent.data.as_ref().map(|v| v.get_value()) {
                Some(val) => {
                    if !Self::validate(&node.type_of, val) {
                        return Value::Null;
                    };
                    self.iter_inner(node, Some(val), parent)
                }
                _ => {
                    if let Some(key) = parent.deferred.get(&node.id) {
                        let value = self.store.get(key);
                        self.iter(node, value)
                    } else {
                        Value::Null
                    }
                }
            },
            None => Value::Null,
        }
    }

    fn iter_inner<'a>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Value<'a>>,
        value: &Data<IoId, OwnedValue>,
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
                            ans.push((
                                child.name.to_owned(),
                                self.iter_inner(child, Some(val), value),
                            ));
                        } else {
                            let current = value
                                .deferred
                                .get(&child.id)
                                .and_then(|io_id| self.store.get(io_id));
                            let value = self.iter(child, current);
                            ans.push((child.name.to_owned(), value));
                        }
                    }
                }
                Value::Object(ans.into())
            }
            Some(Value::Array(arr)) => {
                let mut ans = vec![];
                for val in arr {
                    ans.push(self.iter_inner(node, Some(val), value));
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
    use crate::core::ir::jit::model::FieldId;
    use crate::core::ir::jit::store::{Data, Store};
    use crate::core::ir::jit::synth::Synth;
    use crate::core::ir::IoId;
    use crate::core::valid::Validator;

    const POSTS: &str = r#"
        [
            {"id": 1, "title": "My title", "title":"Hello", "body": "This is my first post.", "userId": 1},
            {"id": 2, "title": "Also My Title", "title":"Alo", "body": "This is my second post.", "userId": 1}
        ]
    "#;

    const TODO: &str = r#"
                {"id": 1, "title": "My title", "completed": false}
        "#;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn synth(query: &str, data: Vec<(IoId, Data<IoId, OwnedValue>)>) -> String {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        let mut store = Store::new();
        let edoc = ExecutionPlanBuilder::new(blueprint, document).build();

        data.into_iter().for_each(|(k, v)| {
            store.insert(k, v);
        });

        let children = edoc.into_children();
        let synth = Synth::new(children.first().unwrap().to_owned(), store);

        serde_json::to_string_pretty(&synth.synthesize()).unwrap()
    }

    #[tokio::test]
    async fn test_synth() {
        let mut store = vec![];

        let mut deferred = HashMap::new();
        deferred.insert(FieldId::new(1), IoId::new(1));
        // Insert Root
        store.push((
            IoId::new(0),
            Data {
                data: None,
                deferred: HashMap::from_iter(vec![(FieldId::new(1), IoId::new(1))].into_iter()),
            },
        ));

        // Insert /posts
        store.push(
            (
                IoId::new(1),
                Data {
                    data: Some(
                        OwnedValue::from_str(
                            r#"[{"id": 1, "title": "My title", "title":"Hello", "body": "This is my first post.", "userId": 1}]"#,
                        )
                            .unwrap(),
                    ),
                    deferred,
                }
            )
        );

        let actual = synth(
            r#"
                query {
                    posts { title body userId }
                }
            "#,
            store,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_users() {
        let mut store = vec![];

        let mut deferred = HashMap::new();
        deferred.insert(FieldId::new(1), IoId::new(1));

        // Insert Root
        store.push((
            IoId::new(0),
            Data {
                data: None,
                deferred: HashMap::from_iter(vec![(FieldId::new(1), IoId::new(1))].into_iter()),
            },
        ));

        // Insert /users
        store.push(
            (
                IoId::new(1),
                Data {
                    data: Some(
                        OwnedValue::from_str(
                            r#"[{"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": 1}]"#,
                        )
                            .unwrap(),
                    ),
                    deferred,
                }
            )
        );

        let actual = synth(
            r#"
                query {
                    users { name address { street } }
                }
            "#,
            store,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_post_id() {
        let mut store = vec![];

        let mut deferred = HashMap::new();
        deferred.insert(FieldId::new(1), IoId::new(1));

        // Insert Root
        store.push((
            IoId::new(0),
            Data {
                data: None,
                deferred: HashMap::from_iter(vec![(FieldId::new(1), IoId::new(1))].into_iter()),
            },
        ));

        let deferred = HashMap::new();
        // Insert /user/:id
        store.push(
            (
                IoId::new(1),
                Data {
                    data: Some(
                        OwnedValue::from_str(
                            r#"{"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": 1}"#,
                        ).unwrap()
                    ),
                    deferred,
                }
            )
        );
        let actual = synth(
            r#"
                query {
                    user(id: 1) { userId name }
                }
            "#,
            store,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_post_id_to_user() {
        let store = vec![
            // Insert Root
            (
                IoId::new(0),
                Data {
                    data: None,
                    deferred: HashMap::from_iter(vec![(FieldId::new(1), IoId::new(1))].into_iter()),
                },
            ),
            // Insert /posts/:id
            (
                IoId::new(1),
                Data {
                    data: Some(
                        OwnedValue::from_str(
                            r#"{"id": 1, "title": "My title", "title":"Hello", "body": "This is my first post.", "userId": 1}"#,
                        )
                            .unwrap(),
                    ),
                    deferred: HashMap::from_iter(
                        vec![
                            (
                                FieldId::new(4),
                                IoId::new(2)
                            )
                        ]
                            .into_iter()
                    ),
                }
            ),
            // Insert /user/:id
            (
                IoId::new(2),
                Data {
                    data: Some(
                        OwnedValue::from_str(
                            r#"{"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": 1}"#,
                        ).unwrap()
                    ),
                    deferred: Default::default(),
                }
            )
        ];

        let actual = synth(
            r#"
                query {
                    post(id: 1) { id title user { name } }
                }
            "#,
            store,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_all_posts_users() {
        let store = vec![
            // Insert Root
            (
                IoId::new(0),
                Data {
                    data: None,
                    deferred: HashMap::from_iter(vec![(FieldId::new(1), IoId::new(1))].into_iter()),
                },
            ),
            // Insert /posts
            (
                IoId::new(1),
                Data {
                    data: Some(
                        OwnedValue::from_str(POSTS).unwrap(),
                    ),
                    deferred: HashMap::from_iter(
                        vec![
                            (
                                FieldId::new(4),
                                IoId::new(2)
                            )
                        ]
                            .into_iter()
                    ),
                }
            ),
            // Insert /user/:id
            (
                IoId::new(2),
                Data {
                    data: Some(
                        OwnedValue::from_str(
                            r#"{"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": 1}"#,
                        ).unwrap()
                    ),
                    deferred: Default::default(),
                }
            )
        ];

        let actual = synth(
            r#"
                query {
                    posts { id title user { name } }
                }
            "#,
            store,
        );

        assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn test_synth_all_posts_users_todos() {
        let store = vec![
            // Insert Root
            (
                IoId::new(0),
                Data {
                    data: None,
                    deferred: HashMap::from_iter(vec![(FieldId::new(1), IoId::new(1))].into_iter()),
                },
            ),
            // Insert /posts
            (
                IoId::new(1),
                Data {
                    data: Some(OwnedValue::from_str(POSTS).unwrap()),
                    deferred: HashMap::from_iter(vec![(FieldId::new(3), IoId::new(2))].into_iter()),
                },
            ),
            // Insert /user/:id
            (
                IoId::new(2),
                Data {
                    data: Some(
                        OwnedValue::from_str(
                            r#"{"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": 1}"#,
                        ).unwrap()
                    ),
                    deferred: HashMap::from_iter(
                        vec![
                            (
                                FieldId::new(5),
                                IoId::new(3)
                            )
                        ].into_iter()
                    ),
                }
            ),
            (
                IoId::new(3),
                Data {
                    data: Some(OwnedValue::from_str(TODO).unwrap()),
                    deferred: Default::default(),
                },
            )
        ];

        let actual = synth(
            r#"
                query {
                    posts { title user { name todo { title completed } } }
                }
            "#,
            store,
        );

        assert_snapshot!(actual);
    }
}
