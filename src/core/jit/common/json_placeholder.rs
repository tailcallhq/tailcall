use std::collections::HashMap;

use serde::Deserialize;

use crate::core::blueprint::Blueprint;
use crate::core::config::{Config, ConfigModule};
use crate::core::jit::builder::Builder;
use crate::core::jit::store::{Data, Store};
use crate::core::jit::synth::Synth;
use crate::core::jit::{OperationPlan, Variables};
use crate::core::json::{JsonLike, JsonObjectLike};
use crate::core::valid::Validator;

/// NOTE: This is a bit of a boilerplate reducing module that is used in tests
/// and benchmarks.
pub struct JsonPlaceholder<Value> {
    test_data: TestData<Value>,
    plan: OperationPlan<Value>,
}

struct TestData<Value> {
    posts: Vec<Value>,
    users: Vec<Value>,
}

impl<'a, Value: JsonLike<'a> + Deserialize<'a> + Clone + 'a> JsonPlaceholder<Value> {
    const POSTS: &'static str = include_str!("posts.json");
    const USERS: &'static str = include_str!("users.json");
    const CONFIG: &'static str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");

    fn plan(query: &str) -> OperationPlan<Value> {
        let config = ConfigModule::from(Config::from_sdl(Self::CONFIG).to_result().unwrap());
        let builder = Builder::new(
            &Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );
        let vars = Variables::new();
        let plan = builder.build(&vars, None).unwrap();

        plan.try_map(Deserialize::deserialize).unwrap()
    }

    pub fn init(query: &str) -> Self {
        let posts = serde_json::from_str::<Vec<Value>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<Value>>(Self::USERS).unwrap();
        let test_data = TestData { posts, users };
        let plan = Self::plan(query);

        JsonPlaceholder { test_data, plan }
    }

    pub fn synth(&'a self) -> Synth<Value> {
        let TestData { posts, users } = &self.test_data;
        let plan = self.plan.clone();

        let user_map = users.iter().fold(HashMap::new(), |mut map, user| {
            let id = user
                .as_object()
                .and_then(|v| v.get_key("id"))
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

        let posts_id = plan.find_field_path(&["posts"]).unwrap().id.to_owned();
        let users_id = plan
            .find_field_path(&["posts", "user"])
            .unwrap()
            .id
            .to_owned();
        let store = [
            (posts_id, Data::Single(Ok(Value::array(posts.clone())))),
            (users_id, Data::Multiple(users)),
        ]
        .into_iter()
        .fold(Store::new(), |mut store, (id, data)| {
            store.set_data(id, data);
            store
        });

        Synth::new(plan, store, Default::default())
    }
}
