pub use serde_json_borrow::*;

use super::model::{Children, Field};
use super::store::{Data, Store};
use crate::core::ir::{CacheKey, EvaluationContext, IoId, ResolverContextLike, IR};

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

    pub fn synthesize<'b, Ctx: ResolverContextLike<'b> + Sync + Send>(
        &self,
        ctx: EvaluationContext<'b, Ctx>,
    ) -> Value {
        self.iter(&self.operation, None, ctx)
    }

    fn is_array(type_of: &crate::core::blueprint::Type, value: &Value) -> bool {
        type_of.is_list() == value.is_array()
    }

    pub fn iter<'a, 'b, Ctx: ResolverContextLike<'b> + Sync + Send>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Data<OwnedValue>>,
        ctx: EvaluationContext<'b, Ctx>,
    ) -> Value<'a> {
        match parent {
            Some(parent) => match parent.data.as_ref().map(|v| v.get_value()) {
                Some(val) => {
                    if !Self::is_array(&node.type_of, val) {
                        return Value::Null;
                    };
                    self.iter_inner(node, Some(val), parent, ctx.clone())
                }
                _ => {
                    match node.ir.as_ref() {
                        Some(IR::IO(io)) => {
                            let key = io.cache_key(&ctx);
                            if let Some(key) = key {
                                let value = self.store.get(&key);
                                if let Some(value) = value {
                                    // check if value exists, else it'll cause stackoverflow
                                    self.iter(node, Some(value), ctx)
                                } else {
                                    // Store does not have data with the IO id, so just return null
                                    Value::Null
                                }
                            } else {
                                Value::Null
                            }
                        }
                        None => Value::Null,
                        _ => {
                            unimplemented!("Need to implement for rest of the IR fields")
                        }
                    }
                }
            },
            None => {
                // TODO: drop the Data struct
                tracing::info!("Hx");
                match node.ir.as_ref() {
                    Some(IR::IO(io)) => {
                        let key = io.cache_key(&ctx);
                        if let Some(key) = key {
                            let value = self.store.get(&key);
                            if let Some(value) = value {
                                tracing::info!("Hx2: {}", node.name);
                                // check if value exists, else it'll cause stackoverflow
                                self.iter(node, Some(value), ctx)
                            } else {
                                tracing::info!("Hx1: {}", node.name);
                                // Store does not have data with the IO id, so just return null
                                Value::Null
                            }
                        } else {
                            Value::Null
                        }
                    }
                    None => Value::Null,
                    _ => {
                        unimplemented!("Need to implement for rest of the IR fields")
                    }
                }
            }
        }
    }

    fn iter_inner<'a, 'b, Ctx: ResolverContextLike<'b> + Sync + Send>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Value<'a>>,
        value: &'a Data<OwnedValue>,
        ctx: EvaluationContext<'b, Ctx>,
    ) -> Value<'a> {
        match parent {
            Some(Value::Object(obj)) => {
                let mut ans = ObjectAsVec::default();
                let children = node.children();
                if children.is_empty() {
                    // if it's a leaf node, then push the value
                    let val = obj.iter().find(|(k, _)| node.name.eq(*k)).map(|(_, v)| v);
                    if let Some(val) = val {
                        ans.insert(node.name.as_str(), val.to_owned());
                    }
                } else {
                    // if it has children, then pick value from obj and pass it to children.
                    for child in children {
                        let val = obj.iter().find(|(k, _)| child.name.eq(*k)).map(|(_, v)| v);
                        if let Some(val) = val {
                            ans.insert(
                                child.name.as_str(),
                                self.iter_inner(child, Some(val), value, ctx.clone()),
                            );
                        } else {
                            let current = match child.ir.as_ref() {
                                Some(IR::IO(io)) => {
                                    io.cache_key(&ctx).and_then(|io_id| self.store.get(&io_id))
                                }
                                _ => None, // TODO: impl for other IRs
                            };
                            let value = self.iter(child, current, ctx.clone());
                            ans.insert(child.name.as_str(), value);
                        }
                    }
                }
                Value::Object(ans)
            }
            Some(Value::Array(arr)) => {
                let mut ans = vec![];
                for val in arr {
                    let cv = serde_json::from_str(val.to_string().as_str()).unwrap_or_default();
                    let ctx = ctx.with_value(cv);
                    ans.push(self.iter_inner(node, Some(val), value, ctx));
                }

                let mut object = ObjectAsVec::default();
                object.insert(node.name.as_str(), Value::Array(ans));
                Value::Object(object)
            }
            Some(val) => val.to_owned(),
            None => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use serde_json_borrow::OwnedValue;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::http::RequestContext;
    use crate::core::ir::jit::builder::ExecutionPlanBuilder;
    use crate::core::ir::jit::store::{Data, Store};
    use crate::core::ir::jit::synth::Synth;
    use crate::core::ir::{EvaluationContext, IoId};
    use crate::core::valid::Validator;

    const POSTS: &str = r#"
        [
            {"id": 1, "title": "My title", "title":"Hello", "body": "This is my first post.", "userId": 1},
            {"id": 2, "title": "Also My Title", "title":"Alo", "body": "This is my second post.", "userId": 1}
        ]
    "#;

    const USERS: &str = r#"
        [
            {"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": 1},
            {"name": "Not Jane Doe", "address": { "street": "Not Kulas Light" }, "userId": 2}
        ]
    "#;

    const USER: &str = r#"
        {"name": "Jane Doe", "address": { "street": "Kulas Light" }, "userId": 1}
    "#;

    const POST: &str = r#"
        {"id": 1, "title": "My title", "title":"Hello", "body": "This is my first post.", "userId": 1}
    "#;

    const TODO: &str = r#"
        [{"id": 1, "title": "My title", "completed": false}]
        "#;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn synth(query: &str, data: Vec<(IoId, Data<OwnedValue>)>) -> String {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        let mut store = Store::new();
        let plan = ExecutionPlanBuilder::new(blueprint, document)
            .build()
            .unwrap();

        data.into_iter().for_each(|(k, v)| {
            store.insert(k, v);
        });

        let children = plan.as_children();
        let synth = Synth::new(children.first().unwrap().to_owned(), store);

        let rt = crate::core::runtime::test::init(None);
        let request_ctx = RequestContext::new(rt);
        let gql_ctx = crate::core::ir::jit::execute::tests::MockGraphqlContext {
            value: Default::default(),
            args: Default::default(),
        };
        let ctx = EvaluationContext::new(&request_ctx, &gql_ctx);

        serde_json::to_string_pretty(&synth.synthesize(ctx)).unwrap()
    }

    #[tokio::test]
    async fn test_synth() {
        let store = vec![
            // Insert /posts
            (
                IoId::new(14498246702353884536),
                Data { data: Some(OwnedValue::from_str(POSTS).unwrap()) },
            ),
        ];

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
        let store = vec![
            // Insert /users
            (
                IoId::new(16572466311295908938),
                Data { data: Some(OwnedValue::from_str(USERS).unwrap()) },
            ),
        ];
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
        let store = vec![
            // Insert /user/:id
            (
                IoId::new(12070278117653709596),
                Data { data: Some(OwnedValue::from_str(USER).unwrap()) },
            ),
        ];
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
            // Insert /posts/:id
            (
                IoId::new(1934628112471474585),
                Data { data: Some(OwnedValue::from_str(POST).unwrap()) },
            ),
            // Insert /user/:id
            (
                IoId::new(12070278117653709596),
                Data { data: Some(OwnedValue::from_str(USER).unwrap()) },
            ),
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
            // Insert /posts
            (
                IoId::new(14498246702353884536),
                Data {
                    data: Some(OwnedValue::from_str(POSTS).unwrap()),
                    // extras: HashMap::from_iter(vec![(FieldId::new(3),
                    // IoId::new(2))].into_iter()),
                },
            ),
            // Insert /user/:id
            (
                IoId::new(12070278117653709596),
                Data {
                    data: Some(OwnedValue::from_str(USER).unwrap()),
                    // extras: Default::default(),
                },
            ),
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
            // Insert /posts
            (
                IoId::new(14498246702353884536),
                Data {
                    data: Some(OwnedValue::from_str(POSTS).unwrap()),
                    // extras: HashMap::from_iter(vec![(FieldId::new(2),
                    // IoId::new(2))].into_iter()),
                },
            ),
            // Insert /user/:id
            (
                IoId::new(12070278117653709596),
                Data {
                    data: Some(OwnedValue::from_str(USER).unwrap()),
                    // extras: HashMap::from_iter(vec![(FieldId::new(4),
                    // IoId::new(3))].into_iter()),
                },
            ),
            (
                IoId::new(14457768933109454390),
                Data {
                    data: Some(OwnedValue::from_str(TODO).unwrap()),
                    // extras: Default::default(),
                },
            ),
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
