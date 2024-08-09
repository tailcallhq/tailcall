use async_graphql::{Name, ServerError};
use async_graphql_value::ConstValue;

use super::{Field, Nested,Request};
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
    is_query: bool,
}
impl<'a, Input: Clone, Output> Context<'a, Input, Output> {
    pub fn new(
        request: &'a Request<Input>,
        is_query: bool,
        field: &'a Field<Nested<Input>, Input>,
    ) -> Self {
        Self { request, is_query, value: None, args: None, field }
    }

    pub fn with_value_and_field(
        &self,
        value: &'a Output,
        field: &'a Field<Nested<Input>, Input>,
    ) -> Self {
        Self { request: self.request, is_query: self.is_query, args: None, value: Some(value), field }
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
            is_query: self.is_query,
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
       self.is_query
    }

    fn add_error(&self, _error: ServerError) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

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
        let ctx: Context<ConstValue, ConstValue> = Context::new(&req, false, &field[0]);
        insta::assert_debug_snapshot!(<Context<_, _> as ResolverContextLike>::field(&ctx).unwrap());
    }
}
