use std::collections::HashMap;

use async_graphql_value::ConstValue;

use crate::core::{blueprint::Blueprint, jit::exec::ExecResult};
use crate::core::config::{Config, ConfigModule};
use crate::core::jit::builder::Builder;
use crate::core::jit::store::{Data, Store};
use crate::core::jit::synth::Synth;
use crate::core::jit::Variables;
use crate::core::json::JsonLike;
use crate::core::valid::Validator;

/// NOTE: This is a bit of a boilerplate reducing module that is used in tests
/// and benchmarks.
pub struct JsonPlaceholder;

impl JsonPlaceholder {
    const POSTS: &'static str = include_str!("posts.json");
    const USERS: &'static str = include_str!("users.json");
    const CONFIG: &'static str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");

    pub fn init(query: &str) -> Synth {
        let posts = serde_json::from_str::<Vec<ConstValue>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<ConstValue>>(Self::USERS).unwrap();

        let user_map = users.iter().fold(HashMap::new(), |mut map, user| {
            let id = if let ConstValue::Object(user) = user {
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
                let user_id = if let ConstValue::Object(post) = post {
                    post.get("userId").and_then(|u| u.as_u64())
                } else {
                    None
                };

                if let Some(user_id) = user_id {
                    if let Some(user) = user_map.get(&user_id) {
                        user.to_owned().to_owned().to_owned()
                    } else {
                        ConstValue::Null
                    }
                } else {
                    ConstValue::Null
                }
            })
            .map(ExecResult::new)
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
        let plan = builder.build(&vars).unwrap();
        let posts_id = plan.find_field_path(&["posts"]).unwrap().id.to_owned();
        let users_id = plan
            .find_field_path(&["posts", "user"])
            .unwrap()
            .id
            .to_owned();
        let store = [
            (posts_id, Data::Single(Ok(ExecResult::new(ConstValue::List(posts))))),
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
