use std::collections::HashMap;

use serde::Deserialize;

use crate::core::blueprint::Blueprint;
use crate::core::config::{Config, ConfigModule};
use crate::core::jit::builder::Builder;
use crate::core::jit::store::Store;
use crate::core::jit::synth::Synth;
use crate::core::jit::{OperationPlan, Variables};
use crate::core::json::{JsonLike, JsonObjectLike};
use crate::core::valid::Validator;

/// NOTE: This is a bit of a boilerplate reducing module that is used in tests
/// and benchmarks.
pub struct JP<Value> {
    test_data: TestData<Value>,
    plan: OperationPlan<Value>,
    vars: Variables<Value>,
}

#[derive(Clone)]
struct TestData<Value> {
    posts: Vec<Value>,
    users: Vec<Value>,
}

struct ProcessedTestData<Value> {
    posts: Value,
    users: Value,
}

impl<'a, Value: JsonLike<'a> + Deserialize<'a> + Clone + 'a> TestData<Value> {
    const POSTS: &'static str = include_str!("posts.json");
    const USERS: &'static str = include_str!("users.json");
    fn init() -> Self {
        let posts = serde_json::from_str::<Vec<Value>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<Value>>(Self::USERS).unwrap();

        TestData { posts, users }
    }

    fn to_processed(&'a self) -> ProcessedTestData<Value> {
        let TestData { posts, users } = self;
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

        let users: Vec<_> = posts
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
            .collect();

        ProcessedTestData {
            posts: Value::array(posts.clone()),
            users: Value::array(users),
        }
    }
}

impl<
        'a,
        Value: Deserialize<'a>
            + Clone
            + 'a
            + JsonLike<'a, JsonObject<'a>: JsonObjectLike<'a, Value = Value>>,
    > JP<Value>
{
    const CONFIG: &'static str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");

    fn plan(query: &str, variables: &Variables<async_graphql::Value>) -> OperationPlan<Value> {
        let config = ConfigModule::from(Config::from_sdl(Self::CONFIG).to_result().unwrap());
        let builder = Builder::new(
            &Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );

        let plan = builder.build(variables, None).unwrap();

        plan.try_map(Deserialize::deserialize).unwrap()
    }

    pub fn init(query: &str, variables: Option<Variables<async_graphql::Value>>) -> Self {
        let vars = variables.unwrap_or_default();

        let test_data = TestData::init();
        let plan = Self::plan(query, &vars);
        let vars = vars.try_map(Deserialize::deserialize).unwrap();

        JP { test_data, plan, vars }
    }

    pub fn synth(&'a self) -> Synth<Value> {
        let ProcessedTestData { posts, users } = self.test_data.to_processed();
        let plan = self.plan.clone();
        let vars = self.vars.clone();

        let posts_id = plan.find_field_path(&["posts"]).unwrap().id.to_owned();
        let users_id = plan
            .find_field_path(&["posts", "user"])
            .unwrap()
            .id
            .to_owned();

        let store = [(posts_id, Ok(posts)), (users_id, Ok(users))]
            .into_iter()
            .fold(Store::new(), |mut store, (id, data)| {
                store.set_data(id, data);
                store
            });

        Synth::new(plan, store, vars)
    }
}
