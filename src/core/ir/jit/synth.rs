pub use serde_json_borrow::*;

use super::model::{Children, Field};
use super::store::{Data, Store};
use crate::core::ir::IoId;

#[allow(unused)]
pub struct Synth {
    operation: Field<Children>,
    store: Store<IoId, OwnedValue>,
}

#[allow(unused)]
impl Synth {
    pub fn new(operation: Field<Children>, store: Store<IoId, OwnedValue>) -> Self {
        Synth { operation, store }
    }

    pub fn synthesize(&self) -> Value<'_> {
        let value = self.store.get(&IoId::new(0));
        self.iter(&self.operation, value)
    }

    pub fn iter<'a>(
        &'a self,
        field: &'a Field<Children>,
        parent: Option<&'a Data<IoId, OwnedValue>>,
    ) -> Value<'a> {
        match parent {
            Some(data) => data
                .value
                .as_ref()
                .map(|a| a.to_owned().into_value())
                .unwrap_or(Value::Null),
            None => Value::Null,
        }
        // match parent {
        //     None => Value::Null,
        //     Some(value) => value,
        // }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use serde_json_borrow::OwnedValue;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::ir::jit::builder::ExecutionPlanBuilder;
    use crate::core::ir::jit::store::{Data, Defer, Store};
    use crate::core::ir::jit::synth::Synth;
    use crate::core::ir::IoId;
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn synth(query: &str) -> String {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        let mut store = Store::new();
        let edoc = ExecutionPlanBuilder::new(blueprint, document).build();

        enum IO {
            Post,
        }

        // Insert Root
        store.insert(
            IoId::new(0),
            Data {
                value: None,
                deferred: vec![Defer {
                    name: "data".to_string(),
                    keys: vec![IoId::new(IO::Post as u64)],
                }],
            },
        );

        // Insert /posts
        store.insert(
            IoId::new(IO::Post as u64),
            Data {
                value: Some(
                    OwnedValue::from_str(
                        r#"[{"title":"Hello", "body": "This is my first post."}]"#,
                    )
                    .unwrap(),
                ),
                deferred: vec![Defer { name: "title".to_string(), keys: vec![] }],
            },
        );

        println!("{:?}", store);

        // Insert /user/:id

        // Synthesize the final value
        let synth = Synth::new(edoc.into_children().first().unwrap().to_owned(), store);

        synth.synthesize().to_string()
    }

    #[tokio::test]
    async fn test_synth() {
        let actual = synth(
            r#"
                query {
                    posts { title }
                }
            "#,
        );

        assert_snapshot!(actual);
    }
}
