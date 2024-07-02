use async_graphql::{Name, Value};
use indexmap::IndexMap;

use super::super::Result;
use super::Synthesizer;
use crate::core::jit::model::{Children, Field};
use crate::core::jit::store::{Data, Store};
use crate::core::jit::ExecutionPlan;
use crate::core::json::JsonLike;

struct Synth {
    selection: Vec<Field<Children>>,
    store: Store<Result<Value>>,
}

impl Synth {
    pub fn new(plan: ExecutionPlan, store: Store<Result<Value>>) -> Self {
        Self { selection: plan.into_children(), store }
    }

    pub fn synthesize(&self) -> Result<Value> {
        let mut data = IndexMap::default();

        for child in self.selection.iter() {
            let val = self.iter(child, None, None)?;
            data.insert(Name::new(child.name.as_str()), val);
        }

        let mut output = IndexMap::default();
        output.insert(Name::new("data"), Value::Object(data));
        Ok(Value::Object(output))
    }

    /// checks if type_of is an array and value is an array
    fn is_array(type_of: &crate::core::blueprint::Type, value: &Value) -> bool {
        type_of.is_list() == value.as_array_ok().is_ok()
    }

    #[inline(always)]
    fn iter<'b>(
        &'b self,
        node: &'b Field<Children>,
        parent: Option<&'b Value>,
        index: Option<usize>,
    ) -> Result<Value> {
        match parent {
            Some(parent) => {
                if !Self::is_array(&node.type_of, parent) {
                    return Ok(Value::Null);
                }
                self.iter_inner(node, parent, index)
            }
            None => {
                // we perform this check to avoid unnecessary hashing

                match self.store.get(&node.id) {
                    Some(val) => {
                        match val {
                            // if index is given, then the data should be a list
                            // if index is not given, then the data should be a value
                            // must return Null in all other cases.
                            Data::Single(val) => {
                                if index.is_some() {
                                    return Ok(Value::Null);
                                }
                                self.iter(node, Some(&val.clone()?), None)
                            }
                            Data::Multiple(list) => {
                                if let Some(i) = index {
                                    match list.get(i) {
                                        Some(val) => self.iter(node, Some(&val.clone()?), None),
                                        None => Ok(Value::Null),
                                    }
                                } else {
                                    Ok(Value::Null)
                                }
                            }
                            Data::Pending => {
                                // TODO: should bailout instead of returning Null
                                Ok(Value::Null)
                            }
                        }
                    }
                    None => {
                        // IR exists, so there must be a value.
                        // if there is no value then we must return Null
                        Ok(Value::Null)
                    }
                }
            }
        }
    }
    #[inline(always)]
    fn iter_inner<'b>(
        &'b self,
        node: &'b Field<Children>,
        parent: &'b Value,
        index: Option<usize>,
    ) -> Result<Value> {
        match parent {
            Value::Object(obj) => {
                let mut ans = IndexMap::default();
                let children = node.children();

                if children.is_empty() {
                    let val = obj.get(node.name.as_str());
                    // if it's a leaf node, then push the value
                    if let Some(val) = val {
                        ans.insert(Name::new(node.name.as_str()), val.to_owned());
                    } else {
                        return Ok(Value::Null);
                    }
                } else {
                    for child in children {
                        let val = obj.get(child.name.as_str());
                        if let Some(val) = val {
                            ans.insert(
                                Name::new(child.name.as_str()),
                                self.iter_inner(child, val, index)?,
                            );
                        } else {
                            ans.insert(
                                Name::new(child.name.as_str()),
                                self.iter(child, None, index)?,
                            );
                        }
                    }
                }
                Ok(Value::Object(ans))
            }
            Value::List(arr) => {
                let mut ans = vec![];
                for (i, val) in arr.iter().enumerate() {
                    let val = self.iter_inner(node, val, Some(i))?;
                    ans.push(val)
                }
                Ok(Value::List(ans))
            }
            val => Ok(val.clone()), // cloning here would be cheaper than cloning whole value
        }
    }
}

pub struct SynthConst {
    plan: ExecutionPlan,
}

impl SynthConst {
    pub fn new(plan: ExecutionPlan) -> Self {
        Self { plan }
    }
}

impl Synthesizer for SynthConst {
    type Value = Result<Value>;

    fn synthesize(self, store: Store<Self::Value>) -> Self::Value {
        Synth::new(self.plan, store).synthesize()
    }
}

#[cfg(test)]
mod tests {

    use async_graphql::Value;

    use super::Synth;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::jit::builder::Builder;
    use crate::core::jit::common::JsonPlaceholder;
    use crate::core::jit::model::FieldId;
    use crate::core::jit::store::{Data, Store};
    use crate::core::valid::Validator;

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
    const USERS: &str = r#"
        [
          {
            "id": 1,
            "name": "Leanne Graham"
          },
          {
            "id": 2,
            "name": "Ervin Howell"
          }
        ]
    "#;

    enum TestData {
        Posts,
        UsersData,
        Users,
        User1,
    }

    impl TestData {
        fn into_value(self) -> Data<Value> {
            match self {
                Self::Posts => Data::Single(serde_json::from_str(POSTS).unwrap()),
                Self::User1 => Data::Single(serde_json::from_str(USER1).unwrap()),
                TestData::UsersData => Data::Multiple(vec![
                    serde_json::from_str(USER1).unwrap(),
                    serde_json::from_str(USER2).unwrap(),
                ]),
                TestData::Users => Data::Single(serde_json::from_str(USERS).unwrap()),
            }
        }
    }

    const CONFIG: &str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");

    fn init(query: &str, store: Vec<(FieldId, Data<Value>)>) -> String {
        let doc = async_graphql::parser::parse_query(query).unwrap();
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let config = ConfigModule::from(config);

        let builder = Builder::new(&Blueprint::try_from(&config).unwrap(), doc);
        let plan = builder.build().unwrap();

        let store = store
            .into_iter()
            .fold(Store::new(plan.size()), |mut store, (id, data)| {
                store.set(id, data.map(Ok));
                store
            });

        let synth = Synth::new(plan, store);
        let val = synth.synthesize().unwrap();

        serde_json::to_string_pretty(&val).unwrap()
    }

    #[test]
    fn test_posts() {
        let store = vec![(FieldId::new(0), TestData::Posts.into_value())];

        let val = init(
            r#"
            query {
                posts { id }
            }
        "#,
            store,
        );
        insta::assert_snapshot!(val);
    }

    #[test]
    fn test_user() {
        let store = vec![(FieldId::new(0), TestData::User1.into_value())];

        let val = init(
            r#"
            query {
                user(id: 1) { id }
            }
        "#,
            store,
        );
        insta::assert_snapshot!(val);
    }

    #[test]
    fn test_nested() {
        let store = vec![
            (FieldId::new(0), TestData::Posts.into_value()),
            (FieldId::new(3), TestData::UsersData.into_value()),
        ];

        let val = init(
            r#"
            query {
                posts { id title user { id name } }
            }
        "#,
            store,
        );
        insta::assert_snapshot!(val);
    }

    #[test]
    fn test_multiple_nested() {
        let store = vec![
            (FieldId::new(0), TestData::Posts.into_value()),
            (FieldId::new(3), TestData::UsersData.into_value()),
            (FieldId::new(6), TestData::Users.into_value()),
        ];

        let val = init(
            r#"
            query {
                posts { id title user { id name } }
                users { id name }
            }
        "#,
            store,
        );
        insta::assert_snapshot!(val)
    }

    #[test]
    fn test_json_placeholder() {
        let synth = JsonPlaceholder::init("{ posts { id title userId user { id name } } }");
        let val = synth.synthesize();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }
}
