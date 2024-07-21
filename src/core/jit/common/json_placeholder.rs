use std::collections::HashMap;

use async_graphql::{Positioned, Value};
use async_graphql_value::ConstValue;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use crate::core::blueprint::Blueprint;
use crate::core::config::{Config, ConfigModule};
use crate::core::jit;
use crate::core::jit::builder::Builder;
use crate::core::jit::model::{FieldId, OperationPlan};
use crate::core::jit::store::{Data, Store};
use crate::core::jit::synth::Synth;
use crate::core::jit::{Error, Variables};
use crate::core::json::{JsonLike, JsonObjectLike};
use crate::core::valid::Validator;

/// NOTE: This is a bit of a boilerplate reducing module that is used in tests
/// and benchmarks.
pub struct JsonPlaceholder;


pub trait SynthExt<Value: for<'a> JsonLike<'a>> {
    fn init(plan: OperationPlan<Value>, data: Vec<(FieldId, Data<Value>)>) -> Self;
    fn synthesize(&'static self) -> Result<Value, Positioned<Error>>;
}

impl SynthExt<ConstValue> for Synth {
    fn init(plan: OperationPlan<ConstValue>, data: Vec<(FieldId, Data<ConstValue>)>) -> Self {
        let store = data
            .into_iter()
            .fold(Store::new(), |mut store, (id, data)| {
                store.set_data(id, data.map(Ok));
                store
            });

        Synth::new(plan, store)
    }

    fn synthesize(&'static self) -> Result<Value, Positioned<Error>> {
        self.synthesize()
    }
}
impl SynthExt<serde_json_borrow::Value<'static>> for Synth<'static> {
    fn init(plan: OperationPlan, data: Vec<(FieldId, Data<BorrowedValue<'static>>)>) -> Self {
        let store = data
            .into_iter()
            .fold(Store::new(), |mut store, (id, data)| {
                store.set_data(id, data);
                store
            });
        SynthBorrow::new(plan, store)
    }

    fn synthesize(&'static self) -> jit::Result<BorrowedValue<'static>> {
        Ok(self.synthesize())
    }
}

struct TestData<T> {
    posts: Vec<T>,
    users: HashMap<usize, Data<T>>,
}

impl JsonPlaceholder {
    const POSTS: &'static str = include_str!("posts.json");
    const USERS: &'static str = include_str!("users.json");
    const CONFIG: &'static str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");


    fn value<Value: for<'json> JsonLike<'json> + DeserializeOwned + Clone + 'static>() -> TestData<Value> {
        let posts = serde_json::from_str::<Vec<Value>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<Value>>(Self::USERS).unwrap();
        let user_map = users.iter().fold(HashMap::new(), |mut map, user| {
            let id = user
                .as_object()
                .and_then(|user| user.get_key("id"))
                .and_then(|u| u.as_u64());
            if let Some(id) = id {
                map.insert(id, user);
            }
            map
        });
        let users: HashMap<_, _> = posts
            .iter()
            .map(|post| {
                let user_id = post
                    .as_object()
                    .and_then(|post| post.get_key("userId"))
                    .and_then(|u| u.as_u64());

                if let Some(user_id) = user_id {
                    if let Some(user) = user_map.get(&user_id) {
                        (*user).to_owned()
                    } else {
                        <Value as JsonLike>::null()
                    }
                } else {
                    <Value as JsonLike>::null()
                }
            })
            .map(Data::Single)
            .enumerate()
            .collect();

        TestData { posts, users }
    }

    fn data<Value: for<'json> JsonLike<'json> + DeserializeOwned + Clone + 'static>(
        plan: &OperationPlan<Value>,
        data: TestData<Value>,
    ) -> Vec<(FieldId, Data<Value>)> {
        let TestData { posts, users } = data;

        let posts_id = plan.find_field_path(&["posts"]).unwrap().id.to_owned();
        let users_id = plan
            .find_field_path(&["posts", "user"])
            .unwrap()
            .id
            .to_owned();
        let store = [
            (
                posts_id,
                Data::Single(<Value as JsonLike>::array(posts)),
            ),
            (users_id, Data::Multiple(users)),
        ];

        store.to_vec()
    }

    fn plan<Value: for<'json> JsonLike<'json> + DeserializeOwned>(query: &str) -> OperationPlan<Value> {
        let config = ConfigModule::from(Config::from_sdl(Self::CONFIG).to_result().unwrap());
        let builder = Builder::new(
            &Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );
        let x = builder.build(&Default::default(), None).unwrap();
        let x = x.try_map(|v| {
            let val = v.into_json()?;
            Ok::<_, anyhow::Error>(serde_json::from_str::<Value>(val.to_string().as_str())?)
        }).unwrap();
        x
    }

    pub fn init(query: &str) -> Synth {
        let posts = serde_json::from_str::<Vec<Value>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<Value>>(Self::USERS).unwrap();

        let user_map = users.iter().fold(HashMap::new(), |mut map, user| {
            let id = if let Value::Object(user) = user {
                user.get("id").and_then(|u| u.as_u64())
            } else {
                None
            };

            if let Some(id) = id {
                map.insert(id, user);
            }
            map
        });

        let users: HashMap<_, _> = posts
            .iter()
            .map(|post| {
                let user_id = if let Value::Object(post) = post {
                    post.get("userId").and_then(|u| u.as_u64())
                } else {
                    None
                };

                if let Some(user_id) = user_id {
                    if let Some(user) = user_map.get(&user_id) {
                        user.to_owned().to_owned().to_owned()
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                }
            })
            .map(Ok)
            .map(Data::Single)
            .enumerate()
            .collect();

        let config = ConfigModule::from(Config::from_sdl(Self::CONFIG).to_result().unwrap());
        let builder = Builder::new(
            &Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );
        let vars = Variables::new();
        let plan = builder.build(&vars, None).unwrap();
        let posts_id = plan.find_field_path(&["posts"]).unwrap().id.to_owned();
        let users_id = plan
            .find_field_path(&["posts", "user"])
            .unwrap()
            .id
            .to_owned();
        let store = [
            (posts_id, Data::Single(Ok(Value::List(posts)))),
            (users_id, Data::Multiple(users)),
        ]
        .into_iter()
        .fold(Store::new(), |mut store, (id, data)| {
            store.set_data(id, data);
            store
        });
        Synth::new(plan, store, vars)
    }
}
