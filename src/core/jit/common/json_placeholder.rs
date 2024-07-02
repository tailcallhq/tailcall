use std::collections::HashMap;

use serde_json_borrow::Value;

use crate::core::blueprint::Blueprint;
use crate::core::config::{Config, ConfigModule};
use crate::core::jit::builder::Builder;
use crate::core::jit::store::{Data, Store};
use crate::core::jit::synth::SynthBorrow;
use crate::core::valid::Validator;

/// NOTE: This is a bit of a boilerplate reducing module that is used in tests
/// and benchmarks.
pub struct JsonPlaceholder;

impl JsonPlaceholder {
    const POSTS: &'static str = include_str!("posts.json");
    const USERS: &'static str = include_str!("users.json");
    const CONFIG: &'static str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");

    pub fn init(query: &str) -> SynthBorrow {
        let posts = serde_json::from_str::<Vec<Value>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<Value>>(Self::USERS).unwrap();

        let user_map = users.iter().fold(HashMap::new(), |mut map, user| {
            let id = user
                .as_object()
                .and_then(|user| user.get("id"))
                .and_then(|u| u.as_u64());

            if let Some(id) = id {
                map.insert(id, user);
            }
            map
        });

        let users: Vec<Value<'static>> = posts
            .iter()
            .map(|post| {
                let user_id = post
                    .as_object()
                    .and_then(|post| post.get("userId").and_then(|u| u.as_u64()));

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
            .collect::<Vec<Value<'static>>>();

        let config = ConfigModule::from(Config::from_sdl(Self::CONFIG).to_result().unwrap());
        let builder = Builder::new(
            &Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );
        let plan = builder.build().unwrap();
        let posts_id = plan.find_field_path(&["posts"]).unwrap().id.to_owned();
        let users_id = plan
            .find_field_path(&["posts", "user"])
            .unwrap()
            .id
            .to_owned();
        let store = [
            (posts_id, Data::Single(Value::Array(posts))),
            (users_id, Data::Multiple(users)),
        ]
        .into_iter()
        .fold(Store::new(plan.size()), |mut store, (id, data)| {
            store.set(id, data);
            store
        });

        SynthBorrow::new(plan, store)
    }
}
