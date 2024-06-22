use serde_json_borrow::{ObjectAsVec, Value};

use crate::core::ir::jit::model::{Children, Field};
use crate::core::ir::jit::store::{Data, Store};

#[allow(unused)]
pub struct Synth {
    operations: Vec<Field<Children>>,
    store: Store,
}

#[allow(unused)]
impl Synth {
    pub fn new(operations: Vec<Field<Children>>, store: Store) -> Self {
        Self { operations, store }
    }
    pub fn synthesize(&self) -> Value {
        let mut data = ObjectAsVec::default();

        for child in self.operations.iter() {
            let val = self.iter(child, None, None);
            data.insert(child.name.as_str(), val);
        }

        let mut output = ObjectAsVec::default();
        output.insert("data", Value::Object(data));
        Value::Object(output)
    }

    /// checks if type_of is an array and value is an array
    fn is_array(type_of: &crate::core::blueprint::Type, value: &Value) -> bool {
        type_of.is_list() == value.is_array()
    }

    #[inline]
    fn iter<'a>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Value>,
        index: Option<usize>,
    ) -> Value {
        match parent {
            Some(parent) => {
                if !Self::is_array(&node.type_of, parent) {
                    return Value::Null;
                }
                self.iter_inner(node, Some(parent), index)
            }
            None => {
                // we perform this check to avoid unnecessary hashing
                if node.ir.is_some() {
                    match self.store.get(&node.id) {
                        Some(val) => {
                            match val {
                                // if index is given, then the data should be a list
                                // if index is not given, then the data should be a value
                                // must return Null in all other cases.
                                Data::Value(val) => {
                                    if index.is_some() {
                                        return Value::Null;
                                    }
                                    self.iter(node, Some(val), None)
                                }
                                Data::List(list) => {
                                    if let Some(i) = index {
                                        match list.get(i) {
                                            Some(val) => self.iter(node, Some(val), None),
                                            None => Value::Null,
                                        }
                                    } else {
                                        Value::Null
                                    }
                                }
                            }
                        }
                        None => {
                            // IR exists, so there must be a value.
                            // if there is no value then we must return Null
                            Value::Null
                        }
                    }
                } else {
                    // either of parent value or IR must exist
                    // if none exist, then we must return Null
                    Value::Null
                }
            }
        }
    }
    #[inline]
    fn iter_inner<'a>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Value>,
        index: Option<usize>,
    ) -> Value {
        match parent {
            Some(Value::Object(obj)) => {
                let mut ans = ObjectAsVec::default();
                let children = node.children();

                if children.is_empty() {
                    let val = obj.get(node.name.as_str());
                    // if it's a leaf node, then push the value
                    if let Some(val) = val {
                        ans.insert(node.name.as_str(), val.to_owned());
                    } else {
                        return Value::Null;
                    }
                } else {
                    for child in children {
                        let val = obj.get(child.name.as_str());
                        if let Some(val) = val {
                            ans.insert(
                                child.name.as_str(),
                                self.iter_inner(child, Some(val), index),
                            );
                        } else {
                            ans.insert(child.name.as_str(), self.iter(child, None, index));
                        }
                    }
                }
                Value::Object(ans)
            }
            Some(Value::Array(arr)) => {
                let mut ans = vec![];
                for (i, val) in arr.iter().enumerate() {
                    let val = self.iter_inner(node, Some(val), Some(i));
                    ans.push(val)
                }
                Value::Array(ans)
            }
            Some(val) => val.clone(), // cloning here would be cheaper than cloning whole value
            None => Value::Null,      // TODO: we can just pass parent value instead of an option
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::ir::common::JsonPlaceholder;
    use crate::core::ir::jit::builder::Builder;
    use crate::core::ir::jit::model::FieldId;
    use crate::core::ir::jit::store::{Data, Store};
    use crate::core::ir::jit::synth::Synth;
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
        fn into_value(self) -> Data<'static> {
            match self {
                Self::Posts => Data::Value(serde_json::from_str(POSTS).unwrap()),
                Self::User1 => Data::Value(serde_json::from_str(USER1).unwrap()),
                TestData::UsersData => Data::List(vec![
                    serde_json::from_str(USER1).unwrap(),
                    serde_json::from_str(USER2).unwrap(),
                ]),
                TestData::Users => Data::Value(serde_json::from_str(USERS).unwrap()),
            }
        }
    }

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn init(query: &str, store: Vec<(FieldId, Data<'static>)>) -> String {
        let doc = async_graphql::parser::parse_query(query).unwrap();
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let config = ConfigModule::from(config);

        let builder = Builder::new(Blueprint::try_from(&config).unwrap(), doc);
        let plan = builder.build().unwrap();

        let store = store
            .into_iter()
            .fold(Store::new(plan.size()), |mut store, (id, data)| {
                store.set(id, data);
                store
            });

        let synth = Synth::new(plan.into_children(), store);
        let val = synth.synthesize();

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
        // FIXME: doesn't work when userId is queried
        let synth = JsonPlaceholder::init("{ posts { id title userId user { id name } } }");
        let val = synth.synthesize();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }
}
