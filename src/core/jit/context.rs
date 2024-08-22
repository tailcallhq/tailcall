use std::sync::{Arc, Mutex, MutexGuard};

use async_graphql::{Name, ServerError};
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use super::error::*;
use super::{Field, Nested, OperationPlan, Positioned};
use crate::core::ir::{ResolverContextLike, SelectionField};

#[derive(Debug, Clone)]
pub struct RequestContext<Input> {
    plan: OperationPlan<Input>,
    errors: Arc<Mutex<Vec<Positioned<Error>>>>,
}

impl<Input> RequestContext<Input> {
    pub fn new(plan: OperationPlan<Input>) -> Self {
        Self { plan, errors: Arc::new(Mutex::new(vec![])) }
    }
    pub fn add_error(&self, new_error: Positioned<Error>) {
        self.errors().push(new_error);
    }
    pub fn plan(&self) -> &OperationPlan<Input> {
        &self.plan
    }
    pub fn errors(&self) -> MutexGuard<Vec<Positioned<Error>>> {
        self.errors.lock().unwrap()
    }
}

/// Rust representation of the GraphQL context available in the DSL
#[derive(Debug, Clone)]
pub struct Context<'a, Input, Output> {
    value: Option<&'a Output>,
    args: Option<indexmap::IndexMap<Name, Input>>,
    // TODO: remove the args, since they're already present inside the fields and add support for
    // default values.
    field: &'a Field<Nested<Input>, Input>,
    request: &'a RequestContext<Input>,
}
impl<'a, Input: Clone, Output> Context<'a, Input, Output> {
    pub fn new(
        field: &'a Field<Nested<Input>, Input>,
        request: &'a RequestContext<Input>,
        is_query: bool,
    ) -> Self {
        Self {
            request,
            value: None,
            args: Self::build_args(field),
            is_query,
            field,
        }
    pub fn with_value_and_field(
        &self,
        value: &'a Output,
        field: &'a Field<Nested<Input>, Input>,
    ) -> Self {
        Self {
            request: self.request,
            args: Self::build_args(field),
            is_query: self.is_query,
            value: Some(value),
            field,
        }
    }

    pub fn with_args(&self, args: indexmap::IndexMap<&str, Input>) -> Self {
        let mut map = indexmap::IndexMap::new();
        for (key, value) in args {
            map.insert(Name::new(key), value);
        }
        Self {
            value: self.value,
            args: Some(map),
            field: self.field,
            request: self.request,
        }
    }

    pub fn value(&self) -> Option<&Output> {
        self.value
    }

    pub fn field(&self) -> &Field<Nested<Input>, Input> {
        self.field
    }

    fn build_args(field: &Field<Nested<Input>, Input>) -> Option<IndexMap<Name, Input>> {
        let mut arg_map = IndexMap::new();

        for arg in field.args.iter() {
            let name = arg.name.as_str();
            let value = arg
                .value
                .clone()
                // TODO: default value resolution should happen in the InputResolver
                .or_else(|| arg.default_value.clone());
            if let Some(value) = value {
                arg_map.insert(Name::new(name), value);
            } else if !arg.type_of.is_nullable() {
                // TODO: throw error here
                todo!()
            }
        }
        Some(arg_map)
    }
}

impl<'a> ResolverContextLike for Context<'a, ConstValue, ConstValue> {
    fn value(&self) -> Option<&ConstValue> {
        self.value
    }

    // TODO: make generic over type of stored values
    fn args(&self) -> Option<&indexmap::IndexMap<Name, ConstValue>> {
        self.args.as_ref()
    }

    fn field(&self) -> Option<SelectionField> {
        Some(SelectionField::from(self.field))
    }

    fn is_query(&self) -> bool {
        self.request.plan().is_query()
    }

    fn add_error(&self, error: ServerError) {
        self.request.add_error(error.into())
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

    use super::{Context, RequestContext};
    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::ir::ResolverContextLike;
    use crate::core::jit::{OperationPlan, Request};
    use crate::core::valid::Validator;

    fn setup(query: &str) -> anyhow::Result<OperationPlan<ConstValue>> {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER)?;
        let config = Config::from_sdl(&sdl).to_result()?;
        let blueprint = Blueprint::try_from(&ConfigModule::from(config))?;
        let request = Request::new(query);
        let plan = request.clone().create_plan(&blueprint)?;
        Ok(plan)
    }

    #[test]
    fn test_field() {
        let plan = setup("query {posts {id title}}").unwrap();
        let field = plan.as_nested();
        let env = RequestContext::new(plan.clone());
        let ctx = Context::<ConstValue, ConstValue>::new(&field[0], &env);
        let expected = <Context<_, _> as ResolverContextLike>::field(&ctx).unwrap();
        insta::assert_debug_snapshot!(expected);
    }

    #[test]
    fn test_is_query() {
        let plan = setup("query {posts {id title}}").unwrap();
        let env = RequestContext::new(plan.clone());
        let ctx = Context::new(&plan.as_nested()[0], &env);
        assert!(ctx.is_query());
    }
}
