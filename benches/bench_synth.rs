use std::collections::HashMap;

use criterion::Criterion;
use serde_json_borrow::Value;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::{Config, ConfigModule};
use tailcall::core::ir::{Builder, Data, FieldId, Store, Synth};
use tailcall::core::valid::Validator;

const CONFIG: &str = include_str!("../src/core/ir/jit/fixtures/jsonplaceholder-mutation.graphql");

// TODO: code is duplicated in synth.rs
struct JsonPlaceholder;

impl JsonPlaceholder {
    const POSTS: &'static str = include_str!("../tailcall-fixtures/fixtures/json/posts.json");
    const USERS: &'static str = include_str!("../tailcall-fixtures/fixtures/json/users.json");

    fn init(query: &str) -> Synth {
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

        let config = ConfigModule::from(Config::from_sdl(CONFIG).to_result().unwrap());
        let builder = Builder::new(
            Blueprint::try_from(&config).unwrap(),
            async_graphql::parser::parse_query(query).unwrap(),
        );
        let plan = builder.build().unwrap();
        let store = [
            (FieldId::new(0), Data::Value(Value::Array(posts))),
            (FieldId::new(3), Data::List(users)),
        ]
        .into_iter()
        .fold(Store::new(plan.size()), |mut store, (id, data)| {
            store.set(id, data);
            store
        });

        Synth::new(plan.into_children(), store)
    }
}

pub fn bench_synth_nested(c: &mut Criterion) {
    c.bench_function("synth_nested", |b| {
        let synth = JsonPlaceholder::init("{ posts { id title user { id name } } }");
        insta::assert_snapshot!(synth.synthesize());
        b.iter(|| {
            let a = synth.synthesize();
            drop(a);
        })
    });
}
