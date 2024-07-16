use async_graphql::{Name, Positioned};
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use super::Synthesizer;
use crate::core::jit::model::{Field, Nested};
use crate::core::jit::store::{Data, Store};
use crate::core::jit::{DataPath, Error, OperationPlan, ValidationError, Variable, Variables};
use crate::core::json::JsonLike;
use crate::core::scalar::get_scalar;

pub struct Synth {
    selection: Vec<Field<Nested<ConstValue>, ConstValue>>,
    store: Store<Result<ConstValue, Positioned<Error>>>,
    variables: Variables<ConstValue>,
}

impl<Extensions, Input> Field<Extensions, Input> {
    #[inline(always)]
    pub fn skip(&self, variables: &Variables<ConstValue>) -> bool {
        let eval = |variable_option: Option<&Variable>,
                    variables: &Variables<ConstValue>,
                    default: bool| {
            match variable_option.map(|a| a.as_str()) {
                Some(name) => variables.get(name).map_or(default, |value| match value {
                    ConstValue::Boolean(b) => *b,
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

impl Synth {
    pub fn new(
        plan: OperationPlan<ConstValue>,
        store: Store<Result<ConstValue, Positioned<Error>>>,
        variables: Variables<ConstValue>,
    ) -> Self {
        Self { selection: plan.into_nested(), store, variables }
    }

    #[inline(always)]
    fn include<T>(&self, field: &Field<T, ConstValue>) -> bool {
        !field.skip(&self.variables)
    }

    pub fn synthesize(&self) -> Result<ConstValue, Positioned<Error>> {
        let mut data = IndexMap::default();

        for child in self.selection.iter() {
            if !self.include(child) {
                continue;
            }
            let val = self.iter(child, None, &DataPath::new())?;
            data.insert(Name::new(child.name.as_str()), val);
        }

        Ok(ConstValue::Object(data))
    }

    /// checks if type_of is an array and value is an array
    fn is_array(type_of: &crate::core::blueprint::Type, value: &ConstValue) -> bool {
        type_of.is_list() == value.as_array().is_some()
    }

    #[inline(always)]
    fn iter<'b>(
        &'b self,
        node: &'b Field<Nested<ConstValue>, ConstValue>,
        parent: Option<&'b ConstValue>,
        data_path: &DataPath,
    ) -> Result<ConstValue, Positioned<Error>> {
        // TODO: this implementation prefer parent value over value in the store
        // that's opposite to the way async_graphql engine works in tailcall
        match parent {
            Some(parent) => {
                if !Self::is_array(&node.type_of, parent) {
                    return Ok(ConstValue::Null);
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
                                _ => return Ok(ConstValue::Null),
                            }
                        }

                        match data {
                            Data::Single(val) => self.iter(node, Some(&val.clone()?), data_path),
                            _ => {
                                // TODO: should bailout instead of returning Null
                                Ok(ConstValue::Null)
                            }
                        }
                    }
                    None => {
                        // IR exists, so there must be a value.
                        // if there is no value then we must return Null
                        Ok(ConstValue::Null)
                    }
                }
            }
        }
    }
    #[inline(always)]
    fn iter_inner<'b>(
        &'b self,
        node: &'b Field<Nested<ConstValue>, ConstValue>,
        parent: &'b ConstValue,
        data_path: &'b DataPath,
    ) -> Result<ConstValue, Positioned<Error>> {
        let include = self.include(node);

        match parent {
            // scalar values should be returned as is
            val if node.is_scalar => {
                let validation = get_scalar(node.type_of.name());

                // TODO: add validation for input type as well. But input types are not checked
                // by async_graphql anyway so it should be done after replacing
                // default engine with JIT
                if validation(val) {
                    Ok(val.clone())
                } else {
                    Err(Positioned {
                        pos: node.pos,
                        node: ValidationError::ScalarInvalid {
                            type_of: node.type_of.name().to_string(),
                            path: node.name.clone(),
                        }
                        .into(),
                    })
                }
            }
            ConstValue::Object(obj) => {
                let mut ans = IndexMap::default();
                if include {
                    if let Some(children) = node.nested() {
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
                    } else {
                        let val = obj.get(node.name.as_str());
                        // if it's a leaf node, then push the value
                        if let Some(val) = val {
                            ans.insert(Name::new(node.name.as_str()), val.to_owned());
                        } else {
                            return Ok(ConstValue::Null);
                        }
                    }
                } else {
                    let val = obj.get(node.name.as_str());
                    // if it's a leaf node, then push the value
                    if let Some(val) = val {
                        ans.insert(Name::new(node.name.as_str()), val.to_owned());
                    } else {
                        return Ok(ConstValue::Null);
                    }
                }
                Ok(ConstValue::Object(ans))
            }
            ConstValue::List(arr) => {
                let mut ans = vec![];
                if include {
                    for (i, val) in arr.iter().enumerate() {
                        let val = self.iter_inner(node, val, &data_path.clone().with_index(i))?;
                        ans.push(val)
                    }
                }
                Ok(ConstValue::List(ans))
            }
            val => Ok(val.clone()), // cloning here would be cheaper than cloning whole value
        }
    }
}

pub struct SynthConst {
    plan: OperationPlan<ConstValue>,
}

impl SynthConst {
    pub fn new(plan: OperationPlan<ConstValue>) -> Self {
        Self { plan }
    }
}

impl Synthesizer for SynthConst {
    type Value = Result<ConstValue, Positioned<Error>>;
    type Variable = ConstValue;

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
        let plan = builder.build(&Variables::new()).unwrap();

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
