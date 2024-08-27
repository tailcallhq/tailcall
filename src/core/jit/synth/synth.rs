use crate::core::ir::TypedValue;
use crate::core::jit::model::{Field, Nested, OperationPlan, Variable, Variables};
use crate::core::jit::store::{Data, DataPath, Store};
use crate::core::jit::{Error, PathSegment, Positioned, ValidationError};
use crate::core::json::{JsonLike, JsonObjectLike};
use crate::core::scalar;

type ValueStore<Value> = Store<Result<Value, Positioned<Error>>>;

pub struct Synth<Value> {
    plan: OperationPlan<Value>,
    store: ValueStore<Value>,
    variables: Variables<Value>,
}

impl<Extensions, Input> Field<Extensions, Input> {
    #[inline(always)]
    pub fn skip<'json, Value: JsonLike<'json>>(&self, variables: &Variables<Value>) -> bool {
        let eval =
            |variable_option: Option<&Variable>, variables: &Variables<Value>, default: bool| {
                variable_option
                    .map(|a| a.as_str())
                    .and_then(|name| variables.get(name))
                    .and_then(|value| value.as_bool())
                    .unwrap_or(default)
            };
        let skip = eval(self.skip.as_ref(), variables, false);
        let include = eval(self.include.as_ref(), variables, true);

        skip == include
    }
}

impl<Value> Synth<Value> {
    #[inline(always)]
    pub fn new(
        plan: OperationPlan<Value>,
        store: ValueStore<Value>,
        variables: Variables<Value>,
    ) -> Self {
        Self { plan, store, variables }
    }
}

impl<'a, Value> Synth<Value>
where
    Value: JsonLike<'a> + Clone + std::fmt::Debug,
    Value::JsonObject<'a>: JsonObjectLike<'a, Value = Value>,
{
    #[inline(always)]
    fn include<T>(&self, field: &Field<T, Value>) -> bool {
        !field.skip(&self.variables)
    }

    #[inline(always)]
    pub fn synthesize(&'a self) -> Result<Value, Positioned<Error>> {
        let mut data = Value::JsonObject::new();

        for child in self.plan.as_nested().iter() {
            if !self.include(child) {
                continue;
            }
            // TODO: in case of error set `child.output_name` to null
            // and append error to response error array
            let val = self.iter(child, None, &DataPath::new())?;

            data.insert_key(&child.output_name, val);
        }

        Ok(Value::object(data))
    }

    #[inline(always)]
    fn iter(
        &'a self,
        node: &'a Field<Nested<Value>, Value>,
        value: Option<&'a Value>,
        data_path: &DataPath,
    ) -> Result<Value, Positioned<Error>> {
        match self.store.get(&node.id) {
            Some(val) => {
                let mut data = val;

                for index in data_path.as_slice() {
                    match data {
                        Data::Multiple(v) => {
                            data = &v[index];
                        }
                        _ => return Ok(Value::null()),
                    }
                }

                match data {
                    Data::Single(result) => {
                        let value = result.as_ref().map_err(Clone::clone)?;

                        if !node.type_of.is_list() == value.as_array().is_some() {
                            return self.node_nullable_guard(node);
                        }
                        self.iter_inner(node, value, data_path)
                    }
                    _ => {
                        // TODO: should bailout instead of returning Null
                        Ok(Value::null())
                    }
                }
            }
            None => match value {
                Some(result) => self.iter_inner(node, result, data_path),
                None => self.node_nullable_guard(node),
            },
        }
    }

    /// This guard ensures to return Null value only if node type permits it, in
    /// case it does not it throws an Error
    fn node_nullable_guard(
        &'a self,
        node: &'a Field<Nested<Value>, Value>,
    ) -> Result<Value, Positioned<Error>> {
        if node.type_of.is_nullable() {
            Ok(Value::null())
        } else {
            // link to GraphQL reference https://spec.graphql.org/October2021/#sec-Handling-Field-Errors
            Err(ValidationError::ValueRequired.into()).map_err(|e| self.to_location_error(e, node))
        }
    }

    #[inline(always)]
    fn iter_inner(
        &'a self,
        node: &'a Field<Nested<Value>, Value>,
        value: &'a Value,
        data_path: &DataPath,
    ) -> Result<Value, Positioned<Error>> {
        // skip the field if field is not included in schema
        if !self.include(node) {
            return Ok(Value::null());
        }

        let eval_result = if value.is_null() {
            // link to GraphQL reference https://spec.graphql.org/October2021/#sec-Handling-Field-Errors
            let is_nullable = match &node.type_of {
                crate::core::blueprint::Type::NamedType { non_null, .. } => !*non_null,
                crate::core::blueprint::Type::ListType { of_type, .. } => of_type.is_nullable(),
            };
            if is_nullable {
                Ok(Value::null())
            } else {
                Err(ValidationError::ValueRequired.into())
            }
        } else if self.plan.field_is_scalar(node) {
            let scalar =
                scalar::Scalar::find(node.type_of.name()).unwrap_or(&scalar::Scalar::Empty);

            // TODO: add validation for input type as well. But input types are not checked
            // by async_graphql anyway so it should be done after replacing
            // default engine with JIT
            if scalar.validate(value) {
                Ok(value.clone())
            } else {
                Err(
                    ValidationError::ScalarInvalid { type_of: node.type_of.name().to_string() }
                        .into(),
                )
            }
        } else if self.plan.field_is_enum(node) {
            if value
                .as_str()
                .map(|v| self.plan.field_validate_enum_value(node, v))
                .unwrap_or(false)
            {
                Ok(value.clone())
            } else {
                Err(
                    ValidationError::EnumInvalid { type_of: node.type_of.name().to_string() }
                        .into(),
                )
            }
        } else {
            match (value.as_array(), value.as_object()) {
                (_, Some(obj)) => {
                    let mut ans = Value::JsonObject::new();

                    let type_name = value.get_type_name().unwrap_or(node.type_of.name());

                    for child in node.nested_iter(type_name) {
                        // all checks for skip must occur in `iter_inner`
                        // and include be checked before calling `iter` or recursing.
                        let include = self.include(child);
                        if include {
                            let val = obj.get_key(child.name.as_str());
                            ans.insert_key(&child.output_name, self.iter(child, val, data_path)?);
                        }
                    }

                    Ok(Value::object(ans))
                }
                (Some(arr), _) => {
                    let mut ans = vec![];
                    for (i, val) in arr.iter().enumerate() {
                        let val = self.iter_inner(node, val, &data_path.clone().with_index(i))?;
                        ans.push(val)
                    }
                    Ok(Value::array(ans))
                }
                _ => Ok(value.clone()),
            }
        };

        eval_result.map_err(|e| self.to_location_error(e, node))
    }

    fn to_location_error(
        &'a self,
        error: Error,
        node: &'a Field<Nested<Value>, Value>,
    ) -> Positioned<Error> {
        // create path from the root to the current node in the fields tree
        let path = {
            let mut path = Vec::new();

            let mut parent = self.plan.find_field(node.id.clone());

            while let Some(field) = parent {
                path.push(PathSegment::Field(field.output_name.to_string()));
                parent = field
                    .parent()
                    .and_then(|id| self.plan.find_field(id.clone()));
            }

            path.reverse();
            path
        };

        Positioned::new(error, node.pos).with_path(path)
    }
}

#[cfg(test)]
mod tests {
    use async_graphql_value::ConstValue;
    use serde::{Deserialize, Serialize};

    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::jit::builder::Builder;
    use crate::core::jit::common::JP;
    use crate::core::jit::model::{FieldId, Variables};
    use crate::core::jit::store::{Data, Store};
    use crate::core::jit::synth::Synth;
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

    #[derive(Clone)]
    enum TestData {
        Posts,
        UsersData,
        Users,
        User1,
    }

    impl TestData {
        fn into_value<'a, Value: Deserialize<'a>>(self) -> Data<Value> {
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

    fn make_store<'a, Value>(query: &str, store: Vec<(FieldId, TestData)>) -> Synth<Value>
    where
        Value: Deserialize<'a> + Serialize + Clone + std::fmt::Debug,
    {
        let store = store
            .into_iter()
            .map(|(id, data)| (id, data.into_value()))
            .collect::<Vec<_>>();

        let doc = async_graphql::parser::parse_query(query).unwrap();
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let config = ConfigModule::from(config);

        let builder = Builder::new(&Blueprint::try_from(&config).unwrap(), doc);
        let plan = builder.build(&Variables::new(), None).unwrap();
        let plan = plan.try_map(Deserialize::deserialize).unwrap();

        let store = store
            .into_iter()
            .fold(Store::new(), |mut store, (id, data)| {
                store.set_data(id, data.map(Ok));
                store
            });
        let vars = Variables::new();

        super::Synth::new(plan, store, vars)
    }

    struct Synths<'a> {
        synth_const: Synth<async_graphql::Value>,
        synth_borrow: Synth<serde_json_borrow::Value<'a>>,
    }

    impl<'a> Synths<'a> {
        fn init(query: &str, store: Vec<(FieldId, TestData)>) -> Self {
            let synth_const = make_store::<ConstValue>(query, store.clone());
            let synth_borrow = make_store::<serde_json_borrow::Value>(query, store.clone());
            Self { synth_const, synth_borrow }
        }
        fn assert(self) {
            let val_const = self.synth_const.synthesize().unwrap();
            let val_const = serde_json::to_string_pretty(&val_const).unwrap();
            let val_borrow = self.synth_borrow.synthesize().unwrap();
            let val_borrow = serde_json::to_string_pretty(&val_borrow).unwrap();
            assert_eq!(val_const, val_borrow);
        }
    }

    #[test]
    fn test_posts() {
        let store = vec![(FieldId::new(0), TestData::Posts)];
        let query = r#"
            query {
                posts { id }
            }
        "#;

        let synths = Synths::init(query, store);
        synths.assert();
    }

    #[test]
    fn test_user() {
        let store = vec![(FieldId::new(0), TestData::User1)];
        let query = r#"
                query {
                    user(id: 1) { id }
                }
            "#;

        let synths = Synths::init(query, store);
        synths.assert();
    }

    #[test]
    fn test_nested() {
        let store = vec![
            (FieldId::new(0), TestData::Posts),
            (FieldId::new(3), TestData::UsersData),
        ];
        let query = r#"
                query {
                    posts { id title user { id name } }
                }
            "#;
        let synths = Synths::init(query, store);
        synths.assert();
    }

    #[test]
    fn test_multiple_nested() {
        let store = vec![
            (FieldId::new(0), TestData::Posts),
            (FieldId::new(3), TestData::UsersData),
            (FieldId::new(6), TestData::Users),
        ];
        let query = r#"
                query {
                    posts { id title user { id name } }
                    users { id name }
                }
            "#;
        let synths = Synths::init(query, store);
        synths.assert();
    }

    #[test]
    fn test_json_placeholder() {
        let jp = JP::init("{ posts { id title userId user { id name } } }", None);
        let synth = jp.synth();
        let val: async_graphql::Value = synth.synthesize().unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }

    #[test]
    fn test_json_placeholder_borrowed() {
        let jp = JP::init("{ posts { id title userId user { id name } } }", None);
        let synth = jp.synth();
        let val: serde_json_borrow::Value = synth.synthesize().unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }
}
