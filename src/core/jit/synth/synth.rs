use std::borrow::Cow;

use crate::core::jit::model::{Field, OperationPlan, Variables};
use crate::core::jit::store::{DataPath, Store};
use crate::core::jit::{Error, PathSegment, Positioned, ValidationError};
use crate::core::json::{JsonLike, JsonObjectLike};

type ValueStore<Value> = Store<Result<Value, Positioned<Error>>>;

pub struct Synth<'a, Value> {
    plan: &'a OperationPlan<Value>,
    store: ValueStore<Value>,
    variables: Variables<Value>,
}

impl<'a, Value> Synth<'a, Value> {
    #[inline(always)]
    pub fn new(
        plan: &'a OperationPlan<Value>,
        store: ValueStore<Value>,
        variables: Variables<Value>,
    ) -> Self {
        Self { plan, store, variables }
    }
}

impl<'a, Value> Synth<'a, Value>
where
    Value: JsonLike<'a> + Clone + std::fmt::Debug,
{
    #[inline(always)]
    fn include(&self, field: &Field<Value>) -> bool {
        !field.skip(&self.variables)
    }

    #[inline(always)]
    pub fn synthesize<Output>(&'a self) -> Result<Output, Positioned<Error>>
    where
        Output: JsonLike<'a>,
    {
        let mut data = Output::JsonObject::with_capacity(self.plan.selection.len());
        let mut path = Vec::new();
        let root_name = self.plan.root_name();

        for child in self.plan.selection.iter() {
            if !self.include(child) {
                continue;
            }
            // TODO: in case of error set `child.output_name` to null
            // and append error to response error array
            let val = self.iter(child, None, &DataPath::new(), &mut path, Some(root_name))?;
            data.insert_key(&child.output_name, val);
        }

        Ok(Output::object(data))
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn iter<Output>(
        &'a self,
        node: &'a Field<Value>,
        value: Option<&'a Value>,
        data_path: &DataPath,
        path: &mut Vec<PathSegment<'a>>,
        root_name: Option<&'a str>,
    ) -> Result<Output, Positioned<Error>>
    where
        Output: JsonLike<'a>,
    {
        path.push(PathSegment::Field(Cow::Borrowed(&node.output_name)));

        let result = match self.store.get(&node.id) {
            Some(value) => {
                let mut value = value.as_ref().map_err(Clone::clone)?;

                for index in data_path.as_slice() {
                    if let Some(arr) = value.as_array() {
                        value = &arr[*index];
                    } else {
                        return Ok(Output::null());
                    }
                }

                if node.type_of.is_list() != value.as_array().is_some() {
                    return self.node_nullable_guard(node, path, None);
                }
                self.iter_inner(node, value, data_path, path)
            }
            None => match value {
                Some(result) => self.iter_inner(node, result, data_path, path),
                None => self.node_nullable_guard(node, path, root_name),
            },
        };

        path.pop();
        result
    }

    /// This guard ensures to return Null value only if node type permits it, in
    /// case it does not it throws an Error
    fn node_nullable_guard<Output>(
        &'a self,
        node: &'a Field<Value>,
        path: &[PathSegment],
        root_name: Option<&'a str>,
    ) -> Result<Output, Positioned<Error>>
    where
        Output: JsonLike<'a>,
    {
        if let Some(root_name) = root_name {
            if node.name.eq("__typename") {
                return Ok(Output::string(Cow::Borrowed(root_name)));
            }
        }
        // according to GraphQL spec https://spec.graphql.org/October2021/#sec-Handling-Field-Errors
        if node.type_of.is_nullable() {
            Ok(Output::null())
        } else {
            Err(ValidationError::ValueRequired.into())
                .map_err(|e| self.to_location_error(e, node, path))
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn iter_inner<Output>(
        &'a self,
        node: &'a Field<Value>,
        value: &'a Value,
        data_path: &DataPath,
        path: &mut Vec<PathSegment<'a>>,
    ) -> Result<Output, Positioned<Error>>
    where
        Output: JsonLike<'a>,
    {
        // skip the field if field is not included in schema
        if !self.include(node) {
            return Ok(Output::null());
        }

        let eval_result = if value.is_null() {
            // check the nullability of this type unwrapping list modifier
            let is_nullable = match &node.type_of {
                crate::core::Type::Named { non_null, .. } => !*non_null,
                crate::core::Type::List { of_type, .. } => of_type.is_nullable(),
            };
            if is_nullable {
                Ok(Output::null())
            } else {
                Err(ValidationError::ValueRequired.into())
            }
        } else if node.scalar.is_some() {
            let scalar = node.scalar.as_ref().unwrap();

            // TODO: add validation for input type as well. But input types are not checked
            // by async_graphql anyway so it should be done after replacing
            // default engine with JIT
            if scalar.validate(value) {
                Ok(Output::clone_from(value))
            } else {
                Err(
                    ValidationError::ScalarInvalid { type_of: node.type_of.name().to_string() }
                        .into(),
                )
            }
        } else if node.is_enum {
            let check_valid_enum = |value: &Value| -> bool {
                value
                    .as_str()
                    .map(|v| self.plan.field_validate_enum_value(node, v))
                    .unwrap_or(false)
            };

            let is_valid_enum = if let Some(vec) = value.as_array() {
                vec.iter().all(check_valid_enum)
            } else {
                check_valid_enum(value)
            };

            if is_valid_enum {
                Ok(Output::clone_from(value))
            } else {
                Err(
                    ValidationError::EnumInvalid { type_of: node.type_of.name().to_string() }
                        .into(),
                )
            }
        } else {
            match (value.as_array(), value.as_object()) {
                (_, Some(obj)) => {
                    let mut fields = Vec::with_capacity(node.selection.len());

                    for child in node
                        .iter()
                        .filter(|field| self.plan.field_is_part_of_value(field, value))
                    {
                        // all checks for skip must occur in `iter_inner`
                        // and include be checked before calling `iter` or recursing.
                        if self.include(child) {
                            let value = if child.name == "__typename" {
                                Output::string(node.value_type(value).into())
                            } else {
                                let val = obj.get_key(child.name.as_str());
                                self.iter(child, val, data_path, path, None)?
                            };
                            fields.push((child.output_name.as_str(), value));
                        }
                    }

                    Ok(Output::object(Output::JsonObject::from_vec(fields)))
                }
                (Some(arr), _) => {
                    let mut ans = Vec::with_capacity(arr.len());
                    for (i, val) in arr.iter().enumerate() {
                        path.push(PathSegment::Index(i));
                        let val =
                            self.iter_inner(node, val, &data_path.clone().with_index(i), path)?;
                        path.pop();
                        ans.push(val);
                    }
                    Ok(Output::array(ans))
                }
                _ => Ok(Output::clone_from(value)),
            }
        };

        eval_result.map_err(|e| self.to_location_error(e, node, path))
    }

    fn to_location_error(
        &'a self,
        error: Error,
        node: &'a Field<Value>,
        path: &[PathSegment],
    ) -> Positioned<Error> {
        Positioned::new(error, node.pos).with_path(
            path.iter()
                .map(|x| match x {
                    PathSegment::Field(cow) => {
                        PathSegment::Field(Cow::Owned(cow.clone().into_owned()))
                    }
                    PathSegment::Index(i) => PathSegment::Index(*i),
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use async_graphql_value::ConstValue;
    use serde::{Deserialize, Serialize};
    use tailcall_valid::Validator;

    use super::ValueStore;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::jit::builder::Builder;
    use crate::core::jit::fixtures::JP;
    use crate::core::jit::model::{FieldId, Variables};
    use crate::core::jit::store::Store;
    use crate::core::jit::synth::Synth;
    use crate::core::jit::OperationPlan;
    use crate::core::json::JsonLike;

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

    fn make_store<'a, Value>(
        query: &str,
        store: Vec<(FieldId, TestData)>,
    ) -> (OperationPlan<Value>, ValueStore<Value>, Variables<Value>)
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

        let builder = Builder::new(&Blueprint::try_from(&config).unwrap(), &doc);
        let plan = builder.build(None).unwrap();
        let plan = plan
            .try_map(|v| {
                // Earlier we hard OperationPlan<ConstValue> which has impl Deserialize
                // but now InputResolver takes OperationPlan<async_graphql_value::Value>
                // and returns OperationPlan<async_graphql_value::Value>.
                // So we need to map Plan to some other value before being able to deserialize
                // it.
                let serde = v.into_json().unwrap();
                Deserialize::deserialize(serde)
            })
            .unwrap();

        let store = store
            .into_iter()
            .fold(Store::new(), |mut store, (id, data)| {
                store.set_data(id, Ok(data));
                store
            });
        let vars = Variables::new();

        (plan, store, vars)
    }

    fn assert_synths(query: &str, store: Vec<(FieldId, TestData)>) {
        let (plan, value_store, vars) = make_store::<ConstValue>(query, store.clone());
        let synth_const = Synth::new(&plan, value_store, vars);
        let (plan, value_store, vars) =
            make_store::<serde_json_borrow::Value>(query, store.clone());
        let synth_borrow = Synth::new(&plan, value_store, vars);

        let val_const: ConstValue = synth_const.synthesize().unwrap();
        let val_const = serde_json::to_string_pretty(&val_const).unwrap();
        let val_borrow: serde_json_borrow::Value = synth_borrow.synthesize().unwrap();
        let val_borrow = serde_json::to_string_pretty(&val_borrow).unwrap();
        assert_eq!(val_const, val_borrow);
    }

    #[test]
    fn test_posts() {
        let store = vec![(FieldId::new(0), TestData::Posts)];
        let query = r#"
            query {
                posts { id }
            }
        "#;

        assert_synths(query, store);
    }

    #[test]
    fn test_user() {
        let store = vec![(FieldId::new(0), TestData::User1)];
        let query = r#"
                query {
                    user(id: 1) { id }
                }
            "#;

        assert_synths(query, store);
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
        assert_synths(query, store);
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
        assert_synths(query, store);
    }

    #[test]
    fn test_json_placeholder() {
        let jp: JP<async_graphql::Value> =
            JP::init("{ posts { id title userId user { id name } } }", None);
        let synth = jp.synth();
        let val: async_graphql::Value = synth.synthesize().unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }

    #[test]
    fn test_json_placeholder_borrowed() {
        let jp: JP<serde_json_borrow::Value> =
            JP::init("{ posts { id title userId user { id name } } }", None);
        let synth = jp.synth();
        let val: serde_json_borrow::Value = synth.synthesize().unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }

    #[test]
    fn test_json_placeholder_typename() {
        let jp: JP<serde_json_borrow::Value> =
            JP::init("{ posts { id __typename user { __typename id } } }", None);
        let synth = jp.synth();
        let val: serde_json_borrow::Value = synth.synthesize().unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }

    #[test]
    fn test_json_placeholder_typename_root_level() {
        let jp: JP<serde_json_borrow::Value> =
            JP::init("{ __typename posts { id user { id }} }", None);
        let synth = jp.synth();
        let val: serde_json_borrow::Value = synth.synthesize().unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&val).unwrap())
    }
}
