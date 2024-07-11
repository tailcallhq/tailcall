use derive_getters::Getters;

use super::Request;
use crate::core::ir::ResolverContextLike;

/// Rust representation of the GraphQL context available in the DSL
#[derive(Getters, Debug)]
pub struct Context<'a, Input, Output> {
    request: &'a Request<Input>,
    value: Option<&'a Output>,
    is_query: bool,
}

impl<'a, Input, Output> Clone for Context<'a, Input, Output> {
    fn clone(&self) -> Self {
        Self {
            request: self.request,
            value: self.value,
            is_query: self.is_query,
        }
    }
}

impl<'a, Input, Output> Context<'a, Input, Output> {
    pub fn new(request: &'a Request<Input>) -> Self {
        Self { request, value: None, is_query: false }
    }

    pub fn with_value(&self, value: &'a Output) -> Self {
        Self {
            request: self.request,
            value: Some(value),
            is_query: self.is_query,
        }
    }

    pub fn with_query(&self, is_query: bool) -> Self {
        Self { request: self.request, value: self.value, is_query }
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
        self.is_query
    }

    fn add_error(&self, _error: async_graphql::ServerError) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use async_graphql::parser::types::OperationType;
    use async_graphql_value::ConstValue;

    use super::Context;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::jit::{ExecutionPlan, Request};
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn setup(query: &str) -> (Request<ConstValue>, ExecutionPlan) {
        let request: Request<ConstValue> = Request::new(query);
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let plan = request.try_new(&blueprint).unwrap();
        (request, plan)
    }

    #[test]
    fn is_query_should_return_true_when_input_is_query_type() {
        let (req, plan) = setup("query {posts {id title}}");
        let ctx: Context<ConstValue, ConstValue> =
            Context::new(&req).with_query(plan.as_nested()[0].0 == OperationType::Query);
        assert!(ctx.is_query())
    }

    #[test]
    fn is_query_should_return_false_when_input_is_mutation_type() {
        let (req, plan) = setup("mutation {createPost(input: {title: \"New Post\"}) {id title}}");
        let ctx: Context<ConstValue, ConstValue> =
            Context::new(&req).with_query(plan.as_nested()[0].0 == OperationType::Query);
        assert!(!ctx.is_query())
    }

    #[test]
    fn should_identify_query_and_mutation_operations() {
        let query = r#"
        query UserPosts {
            posts {
                 id
              userId
              title
            }
            users {
              name
              id
            }
          }
          
          mutation CreatePost {
            createPost(post: {
              userId: 1,
              title: "test-12",
              body: "test-12",
            }) {
              id
              userId
              title
              body
            }
          }
        "#;

        let (req, plan) = setup(query);

        let mut ctx: Context<ConstValue, ConstValue> = Context::new(&req);

        let mut plans = plan.as_nested().to_owned();
        // sort it to avoid the flaky tests.
        plans.sort_by_key(|(op, _)| match op {
            OperationType::Query => 0,
            OperationType::Mutation => 1,
            _ => 3,
        });

        assert!(plans.len() == 2);

        // query
        ctx = ctx.with_query(plans[0].0 == OperationType::Query);
        assert!(ctx.is_query());

        // mutation
        ctx = ctx.with_query(plans[1].0 == OperationType::Query);
        assert!(!ctx.is_query());
    }
}
