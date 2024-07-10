use async_graphql::parser::types::OperationType;
use derive_getters::Getters;

use super::Request;
use crate::core::ir::ResolverContextLike;

/// Rust representation of the GraphQL context available in the DSL
#[derive(Getters, Debug)]
pub struct Context<'a, Input, Output> {
    request: &'a Request<Input>,
    value: Option<&'a Output>,
}

impl<'a, Input, Output> Clone for Context<'a, Input, Output> {
    fn clone(&self) -> Self {
        Self { request: self.request, value: self.value }
    }
}

impl<'a, Input, Output> Context<'a, Input, Output> {
    pub fn new(request: &'a Request<Input>) -> Self {
        Self { request, value: None }
    }

    pub fn with_value(&self, value: &'a Output) -> Self {
        Self { request: self.request, value: Some(value) }
    }
}

impl<'a> ResolverContextLike for Context<'a, async_graphql::Value, async_graphql::Value> {
    fn value(&self) -> Option<&async_graphql::Value> {
        self.value
    }

    fn args(&self) -> Option<&indexmap::IndexMap<async_graphql::Name, async_graphql::Value>> {
        todo!()
    }

    fn field(&self) -> Option<async_graphql::SelectionField> {
        todo!()
    }

    fn is_query(&self) -> bool {
        self.request
            .document
            .as_ref()
            .map_or(false, |exec_document| {
                exec_document
                    .operations
                    .iter()
                    .any(|(_, operation)| operation.node.ty == OperationType::Query)
            })
    }

    fn add_error(&self, _error: async_graphql::ServerError) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

    use super::Context;
    use crate::core::ir::ResolverContextLike;
    use crate::core::jit::Request;

    #[test]
    fn is_query_should_return_true_when_input_is_query_type() {
        let request: Request<ConstValue> = Request::new("query {posts {id title}}");
        let ctx: Context<ConstValue, ConstValue> = Context::new(&request);
        assert!(ctx.is_query())
    }

    #[test]
    fn is_query_should_return_false_when_input_is_mutation_type() {
        let request: Request<ConstValue> =
            Request::new("mutation {createPost(input: {title: \"New Post\"}) {id title}}");
        let ctx: Context<ConstValue, ConstValue> = Context::new(&request);
        assert!(!ctx.is_query())
    }
}
