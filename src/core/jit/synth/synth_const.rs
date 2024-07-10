use async_graphql::{Name, Value};
use indexmap::IndexMap;

use super::super::Result;
use super::Synthesizer;
use crate::core::jit::model::{Field, Nested};
use crate::core::jit::store::{Data, Store};
use crate::core::jit::{DataPath, ExecutionPlan, Variables};
use crate::core::json::JsonLike;

pub struct Synth {
    selection: Vec<Field<Nested>>,
    store: Store<Result<Value>>,
    variables: Variables<async_graphql_value::ConstValue>,
}

impl Synth {
    pub fn new(
        plan: ExecutionPlan,
        store: Store<Result<Value>>,
        variables: Variables<async_graphql_value::ConstValue>,
    ) -> Self {
        Self { selection: plan.into_nested(), store, variables }
    }

    #[inline(always)]
    fn include<T>(&self, field: &Field<T>) -> bool {
        if let Some(include) = &field.include {
            include.include(&self.variables)
        } else {
            true
        }
    }

    pub fn synthesize(&self) -> Result<Value> {
        let mut data = IndexMap::default();

        for child in self.selection.iter() {
            if !self.include(child) {
                continue;
            }
            let val = self.iter(child, None, &DataPath::new())?;
            data.insert(Name::new(child.name.as_str()), val);
        }

        Ok(Value::Object(data))
    }

    /// checks if type_of is an array and value is an array
    fn is_array(type_of: &crate::core::blueprint::Type, value: &Value) -> bool {
        type_of.is_list() == value.as_array_ok().is_ok()
    }

    #[inline(always)]
    fn iter<'b>(
        &'b self,
        node: &'b Field<Nested>,
        parent: Option<&'b Value>,
        data_path: &DataPath,
    ) -> Result<Value> {
        // TODO: this implementation prefer parent value over value in the store
        // that's opposite to the way async_graphql engine works in tailcall
        match parent {
            Some(parent) => {
                if !Self::is_array(&node.type_of, parent) {
                    return Ok(Value::Null);
                }
                self.iter_inner(node, parent, data_path)
            }
            None => {
                // we perform this check to avoid unnecessary hashing

                match self.store.get(&node.id) {
                    Some(val) => {
                        let mut data = val;

                        for index in data_path.as_slice() {
                            match data {
                                Data::Multiple(v) => {
                                    data = &v[index];
                                }
                                _ => return Ok(Value::Null),
                            }
                        }

                        match data {
                            Data::Single(val) => self.iter(node, Some(&val.clone()?), data_path),
                            _ => {
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
        node: &'b Field<Nested>,
        parent: &'b Value,
        data_path: &'b DataPath,
    ) -> Result<Value> {
        let include = self.include(node);

        match parent {
            Value::Object(obj) => {
                let mut ans = IndexMap::default();
                let children = node.nested();
                if include {
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
                            // all checks for skip must occur in `iter_inner`
                            // and include be checked before calling `iter` or recursing.
                            let include = self.include(child);
                            if include {
                                let val = obj.get(child.name.as_str());
                                if let Some(val) = val {
                                    ans.insert(
                                        Name::new(child.name.as_str()),
                                        self.iter_inner(child, val, data_path)?,
                                    );
                                } else {
                                    ans.insert(
                                        Name::new(child.name.as_str()),
                                        self.iter(child, None, data_path)?,
                                    );
                                }
                            }
                        }
                    }
                }
                Ok(Value::Object(ans))
            }
            Value::List(arr) => {
                let mut ans = vec![];
                if include {
                    for (i, val) in arr.iter().enumerate() {
                        let val = self.iter_inner(node, val, &data_path.clone().with_index(i))?;
                        ans.push(val)
                    }
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
    type Variable = Value;

    fn synthesize(
        self,
        store: Store<Self::Value>,
        variables: Variables<Self::Variable>,
    ) -> Self::Value {
        Synth::new(self.plan, store, variables).synthesize()
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
    use crate::core::jit::Variables;
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
                TestData::UsersData => Data::Multiple(
                    vec![
                        Data::Single(serde_json::from_str(USER1).unwrap()),
                        Data::Single(serde_json::from_str(USER2).unwrap()),
                    ]
                    .into_iter()
                    .enumerate()
                    .collect(),
                ),
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
            .fold(Store::new(), |mut store, (id, data)| {
                store.set_data(id, data.map(Ok));
                store
            });
        let vars = Variables::new();
        let synth = Synth::new(plan, store, vars);
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
        let val = synth.synthesize().unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }
}
