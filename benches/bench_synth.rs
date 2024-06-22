use std::collections::HashMap;

use criterion::Criterion;
use serde_json_borrow::Value;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::{Config, ConfigModule};
use tailcall::core::ir::{Builder, Data, FieldId, Store, Synth};
use tailcall::core::valid::Validator;

const CONFIG: &str = include_str!("../src/core/ir/jit/fixtures/jsonplaceholder-mutation.graphql");

pub struct JsonPlaceholder {
    // List of 100 posts
    pub posts: Vec<Value<'static>>,

    // A duplicated List of 100 users one for each post
    pub users: Vec<Value<'static>>,
}

impl Default for JsonPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonPlaceholder {
    const POSTS: &'static str = tailcall_fixtures::json::POSTS;
    const USERS: &'static str = tailcall_fixtures::json::USERS;

    pub fn new() -> Self {
        let posts = serde_json::from_str::<Vec<Value>>(Self::POSTS).unwrap();
        let users = serde_json::from_str::<Vec<Value>>(Self::USERS).unwrap();
        let user_map = users.iter().fold(HashMap::new(), |mut map, n_user| {
            if let Some(user) = n_user.as_object() {
                if let Some(id) = user.get("id").and_then(|u| u.as_u64()) {
                    map.insert(id, n_user);
                }
            }
            map
        });

        let users: Vec<Value> = posts
            .iter()
            .map(|post| {
                if let Some(user_id) = post
                    .as_object()
                    .and_then(|post| post.get("userId").and_then(|u| u.as_u64()))
                {
                    if let Some(user) = user_map.get(&user_id) {
                        return user.to_owned().to_owned();
                    }
                }

                Value::Null
            })
            .collect::<Vec<_>>();

        Self { posts, users }
    }
}
fn create_synthesizer(query: &str) -> Synth {
    let jp = JsonPlaceholder::new();
    let store = vec![
        (FieldId::new(0), Data::Value(Value::Array(jp.posts))),
        (FieldId::new(3), Data::List(jp.users)),
    ];
    let doc = async_graphql::parser::parse_query(query).unwrap();
    let config = Config::from_sdl(CONFIG).to_result().unwrap();
    let config = ConfigModule::from(config);

    let builder = Builder::new(Blueprint::try_from(&config).unwrap(), doc);
    let plan = builder.build().unwrap();
    let size = plan.size();

    let store = store
        .into_iter()
        .fold(Store::new(size), |mut store, (id, data)| {
            store.insert(id, data);
            store
        });

    Synth::new(plan.into_children(), store)
}

pub fn bench_synth_nested(c: &mut Criterion) {
    c.bench_function("synth_nested", |b| {
        let synth = create_synthesizer("{ posts { id title user { id name } } }");
        b.iter(|| {
            let a = synth.synthesize();
            drop(a);
        })
    });
}
