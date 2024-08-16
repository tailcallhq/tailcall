use std::sync::{Arc, Mutex, MutexGuard};

use async_graphql::{Name, ServerError};
use async_graphql_value::ConstValue;

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
    pub fn new(field: &'a Field<Nested<Input>, Input>, env: &'a RequestContext<Input>) -> Self {
        Self { value: None, args: None, field, request: env }
    }

    pub fn with_value_and_field(
        &self,
        value: &'a Output,
        field: &'a Field<Nested<Input>, Input>,
    ) -> Self {
        Self { args: None, value: Some(value), field, request: self.request }
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

    fn setup(query: &str) -> (OperationPlan<ConstValue>, Request<ConstValue>) {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let blueprint = Blueprint::try_from(&ConfigModule::from(config)).unwrap();
        let request = Request::new(query);
        let plan = request.clone().create_plan(&blueprint).unwrap();
        (plan, request)
    }

    #[test]
    fn test_field() {
        let (plan, _) = setup("query {posts {id title}}");
        let field = plan.as_nested();
        let env = RequestContext::new(plan.clone());
        let ctx = Context::<ConstValue, ConstValue>::new(&field[0], &env);
        let expected = <Context<_, _> as ResolverContextLike>::field(&ctx).unwrap();
        insta::assert_debug_snapshot!(expected);
    }
}
