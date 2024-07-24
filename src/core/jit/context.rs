use async_graphql::{Name, ServerError};
use async_graphql_value::ConstValue;

use super::{Field, Nested, OperationPlan, Request};
use crate::core::ir::{ResolverContextLike, SelectionField};

/// Rust representation of the GraphQL context available in the DSL
#[derive(Debug, Clone)]
pub struct Context<'a, Input, Output> {
    request: &'a Request<Input>,
    value: Option<&'a Output>,
    args: Option<indexmap::IndexMap<Name, Input>>,
    // TODO: remove the args, since they're already present inside the fields and add support for
    // default values.
    field: &'a Field<Nested<Input>, Input>,
    plan: &'a OperationPlan<Input>,
}
impl<'a, Input: Clone, Output> Context<'a, Input, Output> {
    pub fn new(
        request: &'a Request<Input>,
        field: &'a Field<Nested<Input>, Input>,
        plan: &'a OperationPlan<Input>,
    ) -> Self {
        Self { request, value: None, args: None, field, plan }
    }

    pub fn with_value_and_field(
        &self,
        value: &'a Output,
        field: &'a Field<Nested<Input>, Input>,
    ) -> Self {
        Self {
            request: self.request,
            args: None,
            value: Some(value),
            field,
            plan: self.plan,
        }
    }

    pub fn with_field(&self, field: &'a Field<Nested<Input>, Input>) -> Self {
        let args = self.args.clone();

        Self {
            request: self.request,
            value: self.value,
            field,
            args,
            plan: self.plan,
        }
    }

    pub fn with_args(&self, args: indexmap::IndexMap<&str, Input>) -> Self {
        let mut map = indexmap::IndexMap::new();
        for (key, value) in args {
            map.insert(Name::new(key), value);
        }
        Self {
            request: self.request,
            value: self.value,
            args: Some(map),
            field: self.field,
            plan: self.plan,
        }
    }

    pub fn value(&self) -> Option<&Output> {
        self.value
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
        self.plan.is_query()
    }

    fn add_error(&self, error: ServerError) {
        self.plan.add_error(error.into())
    }
}

#[cfg(test)]
mod test {
    use super::Context;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::ir::ResolverContextLike;
    use crate::core::jit::{OperationPlan, Request};
    use crate::core::valid::Validator;

    fn setup(
        query: &str,
    ) -> (
        OperationPlan<async_graphql::Value>,
        Request<async_graphql::Value>,
    ) {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let blueprint = Blueprint::try_from(&ConfigModule::from(config)).unwrap();
        let request = Request::new(query);
        let plan = request.clone().create_plan(&blueprint).unwrap();
        (plan, request)
    }

    #[test]
    fn test_field() {
        let (plan, req) = setup("query {posts {id title}}");
        let field = plan.as_nested();
        let ctx: Context<async_graphql::Value, async_graphql::Value> =
            Context::new(&req, &field[0], &plan);
        insta::assert_debug_snapshot!(ctx.field().unwrap());
    }
}
