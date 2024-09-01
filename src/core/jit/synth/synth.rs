use crate::core::jit::model::{Field, Nested, OperationPlan, Variables};
use crate::core::jit::store::{DataPath, Store};
use crate::core::jit::{Error, PathSegment, Positioned, ValidationError};
use crate::core::json::{JsonLike, JsonObjectLike};
use crate::core::scalar;

type ValueStore<Value> = Store<Result<Value, Positioned<Error>>>;

pub struct Synth<Value> {
    plan: OperationPlan<Value>,
    store: ValueStore<Value>,
    variables: Variables<Value>,
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
        let mut path = Vec::new();

        for child in self.plan.as_nested().iter() {
            if !self.include(child) {
                continue;
            }
            // TODO: in case of error set `child.output_name` to null
            // and append error to response error array
            let val = self.iter(child, None, &DataPath::new(), &mut path)?;
            data.insert_key(&child.output_name, val);
        }

        Ok(Value::object(data))
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn iter(
        &'a self,
        node: &'a Field<Nested<Value>, Value>,
        value: Option<&'a Value>,
        data_path: &DataPath,
        path: &mut Vec<PathSegment>,
    ) -> Result<Value, Positioned<Error>> {
        path.push(PathSegment::Field(node.output_name.clone()));

        let result = match self.store.get(&node.id) {
            Some(value) => {
                let mut value = value.as_ref().map_err(Clone::clone)?;

                for index in data_path.as_slice() {
                    if let Some(arr) = value.as_array() {
                        value = &arr[*index];
                    } else {
                        return Ok(Value::null());
                    }
                }

                if node.type_of.is_list() != value.as_array().is_some() {
                    return self.node_nullable_guard(node, path);
                }
                self.iter_inner(node, value, data_path, path)
            }
            None => match value {
                Some(result) => self.iter_inner(node, result, data_path, path),
                None => self.node_nullable_guard(node, path),
            },
        };

        path.pop();
        result
    }

    /// This guard ensures to return Null value only if node type permits it, in
    /// case it does not it throws an Error
    fn node_nullable_guard(
        &'a self,
        node: &'a Field<Nested<Value>, Value>,
        path: &[PathSegment],
    ) -> Result<Value, Positioned<Error>> {
        // according to GraphQL spec https://spec.graphql.org/October2021/#sec-Handling-Field-Errors
        if node.type_of.is_nullable() {
            Ok(Value::null())
        } else {
            Err(ValidationError::ValueRequired.into())
                .map_err(|e| self.to_location_error(e, node, path))
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn iter_inner(
        &'a self,
        node: &'a Field<Nested<Value>, Value>,
        value: &'a Value,
        data_path: &DataPath,
        path: &mut Vec<PathSegment>,
    ) -> Result<Value, Positioned<Error>> {
        // skip the field if field is not included in schema
        if !self.include(node) {
            return Ok(Value::null());
        }

        let eval_result = if value.is_null() {
            // check the nullability of this type unwrapping list modifier
            let is_nullable = match &node.type_of {
                crate::core::Type::Named { non_null, .. } => !*non_null,
                crate::core::Type::List { of_type, .. } => of_type.is_nullable(),
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

                    for child in self.plan.field_iter_only(node, value) {
                        // all checks for skip must occur in `iter_inner`
                        // and include be checked before calling `iter` or recursing.
                        if self.include(child) {
                            let value = if child.name == "__typename" {
                                Value::string(node.value_type(value).into())
                            } else {
                                let val = obj.get_key(child.name.as_str());
                                self.iter(child, val, data_path, path)?
                            };
                            ans.insert_key(&child.output_name, value);
                        }
                    }

                    Ok(Value::object(ans))
                }
                (Some(arr), _) => {
                    let mut ans = vec![];
                    for (i, val) in arr.iter().enumerate() {
                        path.push(PathSegment::Index(i));
                        let val =
                            self.iter_inner(node, val, &data_path.clone().with_index(i), path)?;
                        path.pop();
                        ans.push(val);
                    }
                    Ok(Value::array(ans))
                }
                _ => Ok(value.clone()),
            }
        };

        eval_result.map_err(|e| self.to_location_error(e, node, path))
    }

    fn to_location_error(
        &'a self,
        error: Error,
        node: &'a Field<Nested<Value>, Value>,
        path: &[PathSegment],
    ) -> Positioned<Error> {
        Positioned::new(error, node.pos).with_path(path.to_vec())
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
    use crate::core::jit::store::Store;
    use crate::core::jit::synth::Synth;
    use crate::core::json::JsonLike;
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
        fn into_value<'a, Value: Deserialize<'a> + JsonLike<'a>>(self) -> Value {
            match self {
                Self::Posts => serde_json::from_str(POSTS).unwrap(),
                Self::User1 => serde_json::from_str(USER1).unwrap(),
                TestData::UsersData => Value::array(vec![
                    serde_json::from_str(USER1).unwrap(),
                    serde_json::from_str(USER2).unwrap(),
                ]),
                TestData::Users => serde_json::from_str(USERS).unwrap(),
            }
        }
    }

    const CONFIG: &str = include_str!("../fixtures/jsonplaceholder-mutation.graphql");

    fn make_store<'a, Value>(query: &str, store: Vec<(FieldId, TestData)>) -> Synth<Value>
    where
        Value: Deserialize<'a> + JsonLike<'a> + Serialize + Clone + std::fmt::Debug,
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
                store.set_data(id, Ok(data));
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

    #[test]
    fn test_json_placeholder_typename() {
        let jp = JP::init("{ posts { id __typename user { __typename id } } }", None);
        let synth = jp.synth();
        let val: serde_json_borrow::Value = synth.synthesize().unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }
}
