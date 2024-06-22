use criterion::Criterion;
use serde_json_borrow::Value;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::{Config, ConfigModule};
use tailcall::core::ir::{Builder, Data, FieldId, Store, Synth};
use tailcall::core::valid::Validator;

const POSTS: &str = include_str!("./fixtures/posts.json");
const USERS: &str = include_str!("./fixtures/users.json");

pub(crate) enum TestData {
    Posts,
    UsersData,
}

impl TestData {
    pub(crate) fn into_value(self) -> Data<'static> {
        let posts = serde_json::from_str(POSTS).unwrap();
        let users = serde_json::from_str::<Vec<Value>>(USERS).unwrap();
        let user_0 = &users[0];
        let user_1 = &users[1];

        match self {
            Self::Posts => Data::Value(posts),
            TestData::UsersData => Data::List(vec![user_0.to_owned(), user_1.to_owned()]),
        }
    }
}

const CONFIG: &str = include_str!("../src/core/ir/jit/fixtures/jsonplaceholder-mutation.graphql");

fn create_synthesizer(query: &str) -> Synth {
    let store = vec![
        (FieldId::new(0), TestData::Posts.into_value()),
        (FieldId::new(3), TestData::UsersData.into_value()),
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
