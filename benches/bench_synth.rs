use criterion::Criterion;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::{Config, ConfigModule};
use tailcall::core::ir::{Builder, Data, FieldId, Store, Synth};
use tailcall::core::valid::Validator;

const POSTS: &str = r#"
        [
                {
                    "id": 1,
                    "userId": 1,
                    "title": "Some Title"
                },
                {
                    "id": 2,
                    "userId": 1,
                    "title": "Not Some Title"
                }
        ]
    "#;

const USER1: &str = r#"
        {
                "id": 1,
                "name": "foo"
        }
    "#;

const USER2: &str = r#"
        {
                "id": 2,
                "name": "bar"
        }
    "#;

pub(crate) enum TestData {
    Posts,
    UsersData,
}

impl TestData {
    pub(crate) fn into_value(self) -> Data<'static> {
        match self {
            Self::Posts => Data::Value(serde_json::from_str(POSTS).unwrap()),
            TestData::UsersData => Data::List(vec![
                serde_json::from_str(USER1).unwrap(),
                serde_json::from_str(USER2).unwrap(),
            ]),
        }
    }
}

const CONFIG: &str = include_str!("../src/core/ir/jit/fixtures/jsonplaceholder-mutation.graphql");

pub(crate) fn create_synthesizer(query: &str) -> Synth {
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
