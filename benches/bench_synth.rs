use std::collections::HashMap;

use criterion::Criterion;
use serde_json_borrow::Value;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::{Config, ConfigModule};
use tailcall::core::ir::{Builder, Data, FieldId, Store, Synth};
use tailcall::core::valid::Validator;

const CONFIG: &str = include_str!("../src/core/ir/jit/fixtures/jsonplaceholder-mutation.graphql");

struct JsonPlaceholder {
    // List of 100 posts
    posts: Vec<Value<'static>>,

    // A duplicated List of 100 users one for each post
    users: Vec<Value<'static>>,
}

impl Default for JsonPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonPlaceholder {
    const POSTS: &'static str = tailcall_fixtures::json::POSTS;
    const USERS: &'static str = tailcall_fixtures::json::USERS;

    fn new() -> Self {
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

        Self { posts, users }
    }
}

fn init(query: &str) -> Synth {
    let jp = JsonPlaceholder::new();
    let doc = async_graphql::parser::parse_query(query).unwrap();
    let config = Config::from_sdl(CONFIG).to_result().unwrap();
    let config = ConfigModule::from(config);

    let builder = Builder::new(Blueprint::try_from(&config).unwrap(), doc);
    let plan = builder.build().unwrap();
    let size = plan.size();

    let store = [
        (FieldId::new(0), Data::Value(Value::Array(jp.posts))),
        (FieldId::new(3), Data::List(jp.users)),
    ]
    .into_iter()
    .fold(Store::new(size), |mut store, (id, data)| {
        store.set(id, data);
        store
    });

    Synth::new(plan.into_children(), store)
}

pub fn bench_synth_nested(c: &mut Criterion) {
    c.bench_function("synth_nested", |b| {
        let synth = init("{ posts { id title user { id name } } }");
        b.iter(|| {
            let a = synth.synthesize();
            drop(a);
        })
    });
}
