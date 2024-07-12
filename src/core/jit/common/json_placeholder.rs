use std::collections::HashMap;

use async_graphql::Value;
use async_graphql_value::ConstValue;
use serde::Deserialize;
use serde_json_borrow::Value as BorrowedValue;

use crate::core::blueprint::Blueprint;
use crate::core::config::{Config, ConfigModule};
use crate::core::jit;
use crate::core::jit::builder::Builder;
use crate::core::jit::model::{ExecutionPlan, FieldId, Variables};
use crate::core::jit::store::{Data, Store};
use crate::core::jit::synth::{Synth, SynthBorrow};
use crate::core::json::{JsonLike, JsonObjectLike, JsonT};
use crate::core::valid::Validator;

/// NOTE: This is a bit of a boilerplate reducing module that is used in tests
/// and benchmarks.
pub struct JsonPlaceholder;

pub trait SynthExt<Value: JsonT> {
    fn init(
        plan: ExecutionPlan,
        data: Vec<(FieldId, Data<Value>)>,
        vars: Variables<ConstValue>,
    ) -> Self;
    fn synthesize(&'static self) -> jit::Result<Value>;
}

impl SynthExt<ConstValue> for Synth {
    fn init(
        plan: ExecutionPlan,
        data: Vec<(FieldId, Data<Value>)>,
        vars: Variables<Value>,
    ) -> Self {
        let store = data
            .into_iter()
            .fold(Store::new(), |mut store, (id, data)| {
                store.set_data(id, data.map(Ok));
                store
            });

        Synth::new(plan, store, vars)
    }

    fn synthesize(&'static self) -> jit::Result<Value> {
        self.synthesize()
    }
}

impl SynthExt<BorrowedValue<'static>> for SynthBorrow<'static> {
    fn init(
        plan: ExecutionPlan,
        data: Vec<(FieldId, Data<BorrowedValue<'static>>)>,
        _: Variables<Value>,
    ) -> Self {
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

    fn value<Value: JsonT + Clone + serde::Deserialize<'static>>() -> TestData<Value> {
        let posts = serde_json::from_str::<Vec<Value>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<Value>>(Self::USERS).unwrap();
        let user_map = users.iter().fold(HashMap::new(), |mut map, user| {
            let id =
                Value::object_ok(user).and_then(|v| v.get("id").and_then(|u| u.as_u64_ok().ok()));

            if let Some(id) = id {
                map.insert(id, user);
            }
            map
        });
        let users: HashMap<_, _> = posts
            .iter()
            .map(|post| {
                let user_id = Value::object_ok(post)
                    .and_then(|v| v.get("userId").and_then(|u| u.as_u64_ok().ok()));

                if let Some(user_id) = user_id {
                    if let Some(user) = user_map.get(&user_id) {
                        user.to_owned().to_owned().to_owned()
                    } else {
                        Value::default()
                    }
                } else {
                    Value::default()
                }
            })
            .map(Data::Single)
            .enumerate()
            .collect();
        TestData { posts, users }
    }

    fn plan(query: &str) -> ExecutionPlan {
        let config = ConfigModule::from(Config::from_sdl(Self::CONFIG).to_result().unwrap());
        let builder = Builder::new(
            &Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );
        builder.build().unwrap()
    }

    fn data<Value: JsonT + Clone + serde::Deserialize<'static>>(
        plan: &ExecutionPlan,
    ) -> Vec<(FieldId, Data<Value>)> {
        let TestData { posts, users } = Self::value::<Value>();
        let posts_id = plan.find_field_path(&["posts"]).unwrap().id.to_owned();
        let users_id = plan
            .find_field_path(&["posts", "user"])
            .unwrap()
            .id
            .to_owned();

        let store = [
            (posts_id, Data::Single(<Value as JsonT>::new_array(posts))),
            (users_id, Data::Multiple(users)),
        ];

        store.to_vec()
    }

    pub fn init<Value: JsonT + Deserialize<'static> + Clone, T: SynthExt<Value>>(
        query: &str,
    ) -> Box<T> {
        let plan = Self::plan(query);
        let data = Self::data::<Value>(&plan);
        Box::new(T::init(plan, data, Default::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user() {
        let synth: Box<SynthBorrow> =
            JsonPlaceholder::init("{ posts { id title userId user { id name } } }");
        let val = synth.synthesize();
        println!("{}", val);
    }
}
