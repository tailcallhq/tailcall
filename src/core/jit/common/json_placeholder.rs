use std::any::Any;
use std::collections::HashMap;
use serde::Deserialize;

use serde_json_borrow::Value as BorrowedValue;
use async_graphql_value::ConstValue;

use crate::core::blueprint::Blueprint;
use crate::core::config::{Config, ConfigModule};
use crate::core::jit;
use crate::core::jit::builder::Builder;
use crate::core::jit::model::{ExecutionPlan, FieldId};
use crate::core::jit::store::{Data, Store};
use crate::core::jit::synth::{Synth, SynthBorrow};
use crate::core::json::{JsonLike, Object};
use crate::core::valid::Validator;


pub enum JsonPlaceholderType {
    ConstValue,
    BorrowedValue,
}

/// NOTE: This is a bit of a boilerplate reducing module that is used in tests
/// and benchmarks.
pub struct JsonPlaceholder;

struct TestData<T> {
    posts: Vec<T>,
    users: Vec<T>,
}

impl JsonPlaceholder {
    const POSTS: &'static str = include_str!("posts.json");
    const USERS: &'static str = include_str!("users.json");
    const CONFIG: &'static str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");

    fn value<'a, T: JsonLike + Deserialize<'a> + Clone +'static>() -> TestData<T> {
        let posts = serde_json::from_str::<Vec<T>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<T>>(Self::USERS).unwrap();
        let user_map = users.iter().fold(HashMap::new(), |mut map, user| {
            let id = user
                .as_object_ok()
                .ok()
                .and_then(|user| {
                    user.get_val(&"id")
                })
                .and_then(|u| u.as_u64_ok().ok());
            if let Some(id) = id {
                map.insert(id, user);
            }
            map
        });
        let users = posts
            .iter()
            .map(|post| {
                let user_id = post
                    .as_object_ok()
                    .ok()
                    .and_then(|post| post.get_val(&"userId").and_then(|u| u.as_u64_ok().ok()));

                if let Some(user_id) = user_id {
                    if let Some(user) = user_map.get(&user_id) {
                        (*user).to_owned()
                    } else {
                        <T as JsonLike>::default()
                    }
                } else {
                    <T as JsonLike>::default()
                }
            })
            .collect::<Vec<T>>();

        TestData { posts, users }
    }

    fn plan(query: &str) -> ExecutionPlan {
        let config = ConfigModule::from(Config::from_sdl(Self::CONFIG).to_result().unwrap());
        let builder = Builder::new(
            &Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );
        let plan = builder.build().unwrap();
        plan
    }

    // for Store<Result<ConstValue>> we'll need different type from `T`
    fn data<'a, T1: JsonLike<Output=T1> + Deserialize<'a> + Clone +'static>(plan: &ExecutionPlan, data: TestData<T1>) -> Vec<(FieldId, Data<T1>)> {
        let  TestData { posts, users } = data;

        let posts_id = plan.find_field_path(&["posts"]).unwrap().id.to_owned();
        let users_id = plan
            .find_field_path(&["posts", "user"])
            .unwrap()
            .id
            .to_owned();
        let store = [
            (posts_id, Data::Single(<T1 as JsonLike>::new_array(posts))),
            (users_id, Data::Multiple(users)),
        ];

        store.to_vec()
    }

    pub fn init<T: Any>(query: &str, ty: JsonPlaceholderType) -> T {
        let plan = Self::plan(query);

        match ty {
            JsonPlaceholderType::ConstValue => {
                let data = Self::value::<ConstValue>();
                let store = Self::data(&plan, data)
                    .into_iter()
                    .fold(Store::new(plan.size()), |mut store, (id, data)| {
                        let data: Data<jit::Result<ConstValue>> = match data {
                            Data::Single(single) => {
                                Data::Single(Ok(single))
                            }
                            Data::Multiple(multi) => {
                                Data::Multiple(multi.into_iter().map(Ok).collect())
                            }
                            Data::Pending => { Data::Pending }
                        };
                        store.set(id, data);
                        store
                    });
                Synth::new(plan, store)
            }
            JsonPlaceholderType::BorrowedValue => {
                let data = Self::value::<BorrowedValue>();
                let store = Self::data(&plan, data)
                    .into_iter()
                    .fold(Store::new(plan.size()), |mut store, (id, data)| {
                        store.set(id, data);
                        store
                    });

                SynthBorrow::new(plan, store)
            }
        }
    }
}
