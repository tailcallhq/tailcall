use serde_json_borrow::OwnedValue;
use tokio::sync::Mutex;

use crate::core::ir::{EvaluationContext, EvaluationError, IoId, ResolverContextLike};

use super::model::ExecutionPlan;
use super::store::Store;

#[allow(unused)]
pub struct ExecutionContext<'a, Ctx: ResolverContextLike<'a> + Send + Sync> {
    inner: ExecutionContextInner,
    ctx: EvaluationContext<'a, Ctx>,
}

pub struct ExecutionContextInner {
    pub plan: ExecutionPlan,
    pub store: Mutex<Store<IoId, OwnedValue>>,
}

#[allow(unused)]
impl<'a, Ctx: ResolverContextLike<'a> + Send + Sync> ExecutionContext<'a, Ctx> {
    pub fn new(plan: ExecutionPlan, ctx: EvaluationContext<'a, Ctx>) -> Self {
        Self {
            inner: ExecutionContextInner {
                plan,
                store: Mutex::new(Store::new()),
            },
            ctx,
        }
    }
    pub async fn execute_ir(
        &'a self,
    ) -> Result<(), EvaluationError> {
        super::execute::execute_ir(&self.inner.plan, &self.inner.store, &self.ctx).await
    }
    pub fn into_inner(self) -> ExecutionContextInner {
        self.inner
    }
}

#[cfg(test)]
pub mod tests {
    use async_graphql::{SelectionField, Value};
    use async_graphql_value::Name;
    use indexmap::IndexMap;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::http::RequestContext;
    use crate::core::ir::jit::builder::ExecutionPlanBuilder;
    use crate::core::ir::jit::synth::Synth;
    use crate::core::ir::{EvaluationContext, ResolverContextLike};
    use crate::core::tracing::default_tracing_tailcall;
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    #[derive(Clone)]
    pub struct MockGraphqlContext {
        pub value: Value,
        pub args: IndexMap<Name, Value>,
    }

    impl<'a> ResolverContextLike<'a> for MockGraphqlContext {
        fn value(&'a self) -> Option<&'a Value> {
            Some(&self.value)
        }

        fn args(&'a self) -> Option<&'a IndexMap<Name, Value>> {
            Some(&self.args)
        }

        fn field(&'a self) -> Option<SelectionField> {
            None
        }

        fn is_query(&'a self) -> bool {
            todo!()
        }

        fn add_error(&'a self, _: async_graphql::ServerError) {}
    }

    async fn execute(query: &str) -> String {
        let _guard = tracing::subscriber::set_default(default_tracing_tailcall());

        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        let plan = ExecutionPlanBuilder::new(blueprint, document)
            .build()
            .unwrap();

        let rt = crate::core::runtime::test::init(None);
        let request_ctx = RequestContext::new(rt);
        let gql_ctx = MockGraphqlContext { value: Default::default(), args: Default::default() };
        let ctx = EvaluationContext::new(&request_ctx, &gql_ctx);

        let execution_ctx = super::ExecutionContext::new(plan, ctx.clone());
        execution_ctx.execute_ir().await.unwrap();

        let inner = execution_ctx.into_inner();
        let children = inner.plan.as_children();
        let first = children.first().unwrap().to_owned();

        let synth = Synth::new(first, inner.store.into_inner(), ctx);
        serde_json::to_string_pretty(&synth.synthesize()).unwrap()
    }

    #[tokio::test]
    async fn test_nested() {
        let actual = execute(
            r#"
                query {
                    users { id address { street } }
                }
            "#,
        )
            .await;
        insta::assert_snapshot!(actual);
    }

    #[tokio::test]
    async fn foo() {
        let actual = execute(
            r#"
                query {
                    posts { title userId user { id name todo { completed } } }
                }
            "#,
        )
            .await;
        insta::assert_snapshot!(actual);
    }
}
