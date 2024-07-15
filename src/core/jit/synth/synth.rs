use crate::core::jit::model::{ExecutionPlan, Field, Nested, Variable, Variables};
use crate::core::jit::store::{Data, DataPath, Store};
use crate::core::jit::synth::Synthesizer;
use crate::core::json::{JsonLikeOwned, JsonObjectLike};

// TODO: rename
pub struct AlsoSynth<Value: JsonLikeOwned> {
    plan: ExecutionPlan<Value>,
}

impl<Value: JsonLikeOwned> AlsoSynth<Value> {
    pub fn new(plan: ExecutionPlan<Value>) -> Self {
        Self { plan }
    }
}

impl<Value: JsonLikeOwned + Clone> Synthesizer for AlsoSynth<Value> {
    type Value = Value;
    type Variable = async_graphql_value::ConstValue;

    fn synthesize(self, store: Store<Self::Value>, variables: Variables<Self::Variable>) -> Self::Value {
        let synth = Synth::new(self.plan, store, variables);
        synth.synthesize()
    }
}


pub struct Synth<Value: JsonLikeOwned> {
    selection: Vec<Field<Nested<Value>, Value>>,
    store: Store<Value>,
    variables: Variables<async_graphql_value::ConstValue>,
}


impl<Extensions, Input> Field<Extensions, Input> {
    #[inline(always)]
    pub fn skip(&self, variables: &Variables<async_graphql_value::ConstValue>) -> bool {
        let eval = |variable_option: Option<&Variable>,
                    variables: &Variables<async_graphql_value::ConstValue>,
                    default: bool| {
            match variable_option.map(|a| a.as_str()) {
                Some(name) => variables.get(name).map_or(default, |value| match value {
                    async_graphql_value::ConstValue::Boolean(b) => *b,
                    _ => default,
                }),
                None => default,
            }
        };
        let skip = eval(self.skip.as_ref(), variables, false);
        let include = eval(self.include.as_ref(), variables, true);

        skip == include
    }
}

impl<Value: JsonLikeOwned + Clone> Synth<Value> {
    #[inline(always)]
    pub fn new(
        plan: ExecutionPlan<Value>,
        store: Store<Value>,
        variables: Variables<async_graphql_value::ConstValue>,
    ) -> Self {
        Self {
            selection: plan.into_nested(),
            store,
            variables,
        }
    }

    #[inline(always)]
    fn include<T>(&self, field: &Field<T, Value>) -> bool {
        !field.skip(&self.variables)
    }

    #[inline(always)]
    pub fn synthesize(&self) -> Value {
        let mut data = Value::JsonObject::new();

        for child in self.selection.iter() {
            if !self.include(child) {
                continue;
            }
            let val = self.iter(child, None, &DataPath::new());
            data = data.insert_key(child.name.as_str(), val);
        }

        Value::object(data)
    }

    /// checks if type_of is an array and value is an array
    #[inline(always)]
    fn is_array(type_of: &crate::core::blueprint::Type, value: &Value) -> bool {
        type_of.is_list() == value.as_array().is_some()
    }

    #[inline(always)]
    fn iter<'b>(
        &'b self,
        node: &'b Field<Nested<Value>, Value>,
        parent: Option<&'b Value>,
        data_path: &DataPath,
    ) -> Value {
        // TODO: this implementation prefer parent value over value in the store
        // that's opposite to the way async_graphql engine works in tailcall
        match parent {
            Some(parent) => {
                if !Self::is_array(&node.type_of, parent) {
                    return Value::null();
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
                                _ => return Value::null(),
                            }
                        }

                        match data {
                            Data::Single(val) => self.iter(node, Some(&val.clone()), data_path),
                            _ => {
                                // TODO: should bailout instead of returning Null
                                Value::null()
                            }
                        }
                    }
                    None => {
                        // IR exists, so there must be a value.
                        // if there is no value then we must return Null
                        Value::null()
                    }
                }
            }
        }
    }
    #[inline(always)]
    fn iter_inner<'b>(
        &'b self,
        node: &'b Field<Nested<Value>, Value>,
        parent: &'b Value,
        data_path: &'b DataPath,
    ) -> Value {
        let include = self.include(node);
        match (parent.as_array(), parent.as_object()) {
            (_, Some(obj)) => {
                let mut ans = Value::JsonObject::new();
                if include {
                    if let Some(children) = node.nested() {
                        for child in children {
                            // all checks for skip must occur in `iter_inner`
                            // and include be checked before calling `iter` or recursing.
                            let include = self.include(child);
                            if include {
                                let val = obj.get_key(child.name.as_str());
                                if let Some(val) = val {
                                    ans = ans.insert_key(
                                        child.name.as_str(),
                                        self.iter_inner(child, val, data_path),
                                    );
                                } else {
                                    ans = ans.insert_key(
                                        child.name.as_str(),
                                        self.iter(child, None, data_path),
                                    );
                                }
                            }
                        }
                    } else {
                        let val = obj.get_key(node.name.as_str());
                        // if it's a leaf node, then push the value
                        if let Some(val) = val {
                            ans = ans.insert_key(node.name.as_str(), val.to_owned());
                        } else {
                            return Value::null();
                        }
                    }
                } else {
                    let val = obj.get_key(node.name.as_str());
                    // if it's a leaf node, then push the value
                    if let Some(val) = val {
                        ans = ans.insert_key(node.name.as_str(), val.to_owned());
                    } else {
                        return Value::null();
                    }
                }
                Value::object(ans)
            }
            (Some(arr), _) => {
                let mut ans = vec![];
                if include {
                    for (i, val) in arr.iter().enumerate() {
                        let val = self.iter_inner(node, val, &data_path.clone().with_index(i));
                        ans.push(val)
                    }
                }
                Value::array(ans)
            }
            _ => parent.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::jit::model::{FieldId, Variables};
    use crate::core::jit::store::{Data, Store};
    use async_graphql::Value;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::jit::builder::Builder;
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
        let plan = builder.build(&Variables::new()).unwrap();

        let store = store
            .into_iter()
            .fold(Store::new(), |mut store, (id, data)| {
                store.set_data(id, data);
                store
            });
        let vars = Variables::new();
        let synth = super::Synth::new(plan, store, vars);
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
}