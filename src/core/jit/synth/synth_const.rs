use async_graphql::{Name, PathSegment};
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use super::Synthesizer;
use crate::core::jit::store::{Data, Store};
use crate::core::jit::{
    model::{Field, Nested},
    LocationError,
};
use crate::core::jit::{DataPath, Error, OperationPlan, ValidationError, Variable, Variables};
use crate::core::json::JsonLike;
use crate::core::scalar::get_scalar;

pub struct Synth {
    plan: OperationPlan<ConstValue>,
    store: Store<Result<ConstValue, Error>>,
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
        store: Store<Result<ConstValue, Error>>,
        variables: Variables<ConstValue>,
    ) -> Self {
        Self { plan, store, variables }
    }

    #[inline(always)]
    fn include<T>(&self, field: &Field<T, ConstValue>) -> bool {
        !field.skip(&self.variables)
    }

    pub fn synthesize(&self) -> Result<ConstValue, LocationError<Error>> {
        let mut data = IndexMap::default();

        for child in self.plan.as_nested().iter() {
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
    ) -> Result<ConstValue, LocationError<Error>> {
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
                            Data::Single(val) => self.iter(
                                node,
                                Some(
                                    &val.clone()
                                        .map_err(|error| self.to_location_error(error, node))?,
                                ),
                                data_path,
                            ),
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
    ) -> Result<ConstValue, LocationError<Error>> {
        let include = self.include(node);

        match parent {
            ConstValue::Null => {
                if node.type_of.is_nullable() {
                    Ok(ConstValue::Null)
                } else {
                    Err(ValidationError::ValueRequired.into())
                }
            }
            // scalar values should be returned as is
            val if self.plan.field_is_scalar(node) => {
                let validation = get_scalar(node.type_of.name());

                // TODO: add validation for input type as well. But input types are not checked
                // by async_graphql anyway so it should be done after replacing
                // default engine with JIT
                if validation(val) {
                    Ok(val.clone())
                } else {
                    Err(
                        ValidationError::ScalarInvalid { type_of: node.type_of.name().to_string() }
                            .into(),
                    )
                }
            }
            val if self.plan.field_is_enum(node) => {
                if val
                    .as_str()
                    .map(|v| self.plan.field_validate_enum_value(node, v))
                    .unwrap_or(false)
                {
                    Ok(val.clone())
                } else {
                    Err(
                        ValidationError::EnumInvalid { type_of: node.type_of.name().to_string() }
                            .into(),
                    )
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
        .map_err(|error| self.to_location_error(error, node))
    }

    fn to_location_error(
        &self,
        error: Error,
        node: &Field<Nested<ConstValue>, ConstValue>,
    ) -> LocationError<Error> {
        // create path from the root to the current node in the fields tree
        let path = {
            let mut path = Vec::new();

            let mut parent = self.plan.find_field(node.id.clone());

            while let Some(field) = parent {
                path.push(PathSegment::Field(field.name.to_string()));
                parent = field
                    .parent()
                    .and_then(|id| self.plan.find_field(id.clone()));
            }

            path.reverse();
            path
        };

        LocationError { error, pos: node.pos, path }
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
    type Value = Result<ConstValue, Error>;
    type Output = Result<ConstValue, LocationError<Error>>;
    type Variable = ConstValue;

    fn synthesize(
        self,
        store: Store<Self::Value>,
        variables: Variables<Self::Variable>,
    ) -> Self::Output {
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
        let plan = builder.build(&Variables::new(), None).unwrap();

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
