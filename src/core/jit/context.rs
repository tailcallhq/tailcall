use std::sync::{Arc, Mutex, MutexGuard};

use async_graphql::{Name, ServerError};
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use super::error::*;
use super::{Field, OperationPlan, Positioned};
use crate::core::ir::{ResolverContextLike, SelectionField};

#[derive(Debug)]
pub struct RequestContext<'a, Input> {
    plan: &'a OperationPlan<Input>,
    errors: Arc<Mutex<Vec<Positioned<Error>>>>,
}

impl<'a, Input> RequestContext<'a, Input> {
    pub fn new(plan: &'a OperationPlan<Input>) -> Self {
        Self { plan, errors: Arc::new(Mutex::new(vec![])) }
    }
    pub fn add_error(&self, new_error: Positioned<Error>) {
        self.errors().push(new_error);
    }
    pub fn plan(&self) -> &OperationPlan<Input> {
        self.plan
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
    field: &'a Field<Input>,
    request: &'a RequestContext<'a, Input>,
}
impl<'a, Input: Clone, Output> Context<'a, Input, Output> {
    pub fn new(field: &'a Field<Input>, request: &'a RequestContext<Input>) -> Self {
        Self { request, value: None, args: Self::build_args(field), field }
    }

    pub fn with_value(&self, value: &'a Output) -> Self {
        Self {
            request: self.request,
            // TODO: no need to build again?
            args: Self::build_args(self.field),
            value: Some(value),
            field: self.field,
        }
    }

    pub fn with_value_and_field(&self, value: &'a Output, field: &'a Field<Input>) -> Self {
        Self {
            request: self.request,
            args: Self::build_args(field),
            value: Some(value),
            field,
        }
    }

    pub fn value(&self) -> Option<&Output> {
        self.value
    }

    pub fn field(&self) -> &Field<Input> {
        self.field
    }

    fn build_args(field: &Field<Input>) -> Option<IndexMap<Name, Input>> {
        let mut arg_map = IndexMap::new();

        for arg in field.args.iter() {
            let name = arg.name.as_str();
            let value = arg.value.clone();
            if let Some(value) = value {
                arg_map.insert(Name::new(name), value);
            }
        }
        Some(arg_map)
    }
}

impl ResolverContextLike for Context<'_, ConstValue, ConstValue> {
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
    use tailcall_valid::Validator;

    use super::{Context, RequestContext};
    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::ir::ResolverContextLike;
    use crate::core::jit::transform::InputResolver;
    use crate::core::jit::{OperationPlan, Request};

    fn setup(query: &str) -> anyhow::Result<OperationPlan<ConstValue>> {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER)?;
        let config = Config::from_sdl(&sdl).to_result()?;
        let blueprint = Blueprint::try_from(&ConfigModule::from(config))?;
        let request = Request::new(query);
        let plan = request.clone().create_plan(&blueprint)?;
        let input_resolver = InputResolver::new(plan);
        let plan = input_resolver.resolve_input(&Default::default()).unwrap();

        Ok(plan)
    }

    #[test]
    fn test_field() {
        let plan = setup("query {posts {id title}}").unwrap();
        let field = &plan.selection;
        let env = RequestContext::new(&plan);
        let ctx = Context::<ConstValue, ConstValue>::new(&field[0], &env);
        let expected = <Context<_, _> as ResolverContextLike>::field(&ctx).unwrap();
        insta::assert_debug_snapshot!(expected);
    }

    #[test]
    fn test_is_query() {
        let plan = setup("query {posts {id title}}").unwrap();
        let env = RequestContext::new(&plan);
        let ctx = Context::new(&plan.selection[0], &env);
        assert!(ctx.is_query());
    }
}
