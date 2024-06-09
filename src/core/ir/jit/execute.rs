use serde_json_borrow::OwnedValue;
use tokio::sync::Mutex;

use crate::core::ir::{CallId, EvaluationContext, EvaluationError, IR, ResolverContextLike};
use crate::core::ir::jit::model::ExecutionPlan;
use crate::core::ir::jit::store::Store;

#[allow(unused)]
pub async fn execute_ir<'a, Ctx: ResolverContextLike<'a>>(
    plan: &'a ExecutionPlan,
    store: &'a Mutex<Store<CallId, OwnedValue>>,
    ctx: EvaluationContext<'a, Ctx>,
) -> Result<(), EvaluationError> {

}

#[cfg(test)]
mod tests {
    use async_graphql::{SelectionField, Value};
    use async_graphql_value::Name;
    use indexmap::IndexMap;
    use tokio::sync::Mutex;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::http::RequestContext;
    use crate::core::ir::jit::builder::ExecutionPlanBuilder;
    use crate::core::ir::jit::execute::execute_ir;
    use crate::core::ir::jit::store::Store;
    use crate::core::ir::jit::synth::Synth;
    use crate::core::ir::{EvaluationContext, ResolverContextLike};
    use crate::core::tracing::default_tracing_tailcall;
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    #[derive(Clone)]
    struct MockGraphqlContext {
        value: Value,
        args: IndexMap<Name, Value>,
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

        let store = Mutex::new(Store::new());
        let request_ctx = RequestContext::new(rt);
        let gql_ctx = MockGraphqlContext { value: Default::default(), args: Default::default() };
        let ctx = EvaluationContext::new(&request_ctx, &gql_ctx);
        execute_ir(&plan, &store, ctx).await.unwrap();
        let store = store.into_inner();
        // tracing::info!("{:#?}", store);
        let children = plan.as_children();
        let synth = Synth::new(children.first().unwrap().to_owned(), store);
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
                    posts { id user { id name } }
                }
            "#,
        )
            .await;
        insta::assert_snapshot!(actual);
    }
}
