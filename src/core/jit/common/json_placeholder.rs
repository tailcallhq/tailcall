use std::collections::HashMap;
use serde::de::DeserializeOwned;
use crate::core::blueprint::Blueprint;
use crate::core::config::{Config, ConfigModule};
use crate::core::jit::builder::Builder;
use crate::core::jit::store::{Data, Store};
use crate::core::jit::synth::Synth;
use crate::core::jit::Variables;
use crate::core::json::{JsonLikeOwned, JsonObjectLike};
use crate::core::valid::Validator;

/// NOTE: This is a bit of a boilerplate reducing module that is used in tests
/// and benchmarks.
pub struct JsonPlaceholder;

impl JsonPlaceholder {
    const POSTS: &'static str = include_str!("posts.json");
    const USERS: &'static str = include_str!("users.json");
    const CONFIG: &'static str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");


/*
    fn value<Value: JsonLike<'static> + Deserialize<'static> + Clone + 'static>() -> TestData<Value> {
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

    fn plan(query: &str) -> OperationPlan<async_graphql::Value> {
        let config = ConfigModule::from(Config::from_sdl(Self::CONFIG).to_result().unwrap());
        let builder = Builder::new(
            &Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );
        let plan = builder.build(&Default::default(), None).unwrap();
        /*let plan = plan.try_map(|v| {
            let val = v.into_json()?;
            let val = serde_json::from_value(val)?;
            Ok::<Value, anyhow::Error>(val)
        }).unwrap();*/

        plan
    }

    fn data<'a, Value: JsonLike<'a> + Deserialize<'a> + Clone + 'static>(
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
*/
    pub fn init<Value: JsonLikeOwned + DeserializeOwned + Clone + 'static>(query: &str) -> Synth<Value> {
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
                    .and_then(|v| v.get_key("userId"))
                    .and_then(|u| u.as_u64());

                if let Some(user_id) = user_id {
                    if let Some(user) = user_map.get(&user_id) {
                        user.to_owned().to_owned().to_owned()
                    } else {
                        Value::null()
                    }
                } else {
                    Value::null()
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
            (posts_id, Data::Single(Ok(Value::array(posts)))),
            (users_id, Data::Multiple(users)),
        ]
        .into_iter()
        .fold(Store::new(), |mut store, (id, data)| {
            store.set_data(id, data);
            store
        });
        let plan = plan.try_map(|v| {
            let val = v.into_json()?;
            let val = serde_json::from_value(val)?;
            Ok::<Value, anyhow::Error>(val)
        }).unwrap();
        Synth::new(plan, store, vars)
    }
}
