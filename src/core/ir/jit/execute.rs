use serde_json_borrow::{OwnedValue, Value};
use tokio::sync::Mutex;

use crate::core::ir::jit::model::{Children, ExecutionPlan, Field};
use crate::core::ir::jit::store::Store;
use crate::core::ir::{
    CacheKey, Eval, EvaluationContext, EvaluationError, IoId, ResolverContextLike, IR,
};

struct IO {
    data: OwnedValue,
    id: IoId,
}
impl IO {
    fn new(value: OwnedValue, id: IoId) -> Self {
        Self { data: value, id }
    }
}

#[allow(unused)]
pub async fn execute_ir<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    plan: &'a ExecutionPlan,
    store: &'a Mutex<Store<IoId, OwnedValue>>,
    ctx: &'a EvaluationContext<'a, Ctx>,
) -> Result<(), EvaluationError> {
    let mut store = store.lock().await;
    for child in plan.as_children() {
        // iterate over root nodes and resolve all children
        iter(&mut store, child, ctx.clone(), None).await?;
    }
    Ok(())
}

#[allow(clippy::multiple_bound_locations)]
#[async_recursion::async_recursion]
async fn iter<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    store: &mut Store<IoId, OwnedValue>,
    node: &'a Field<Children>,
    mut ctx: EvaluationContext<'a, Ctx>,
    parent_val: Option<&Value>,
) -> Result<(), EvaluationError> {
    if let Some(ir) = node.ir.as_ref() {
        match parent_val {
            Some(Value::Array(array)) => {
                for val in array {
                    iter(store, node, ctx.clone(), Some(val)).await?
                }
                // we just need to execute inner values.
                return Ok(());
            }
            Some(val) => {
                // TODO: maybe avoid serialization/deserialization here
                let val = serde_json::from_str(val.to_string().as_str()).map_err(|e| {
                    EvaluationError::DeserializeError(format!(
                        "Failed to deserialize ConstValue: {}",
                        e
                    ))
                })?;
                ctx = ctx.with_value(val);
            }
            _ => (),
        }

        let io = execute(ir, ctx.clone()).await?;
        let call_id = io.id;

        for child in node.children() {
            if child.ir.is_some() {
                iter(store, child, ctx.clone(), Some(io.data.get_value())).await?;
            }
        }

        store.insert(call_id, io.data);
    }

    Ok(())
}

async fn execute<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    ir: &'a IR,
    ctx: EvaluationContext<'a, Ctx>,
) -> Result<IO, EvaluationError> {
    // TODO: should implement some kind of key for all fields of IR
    match ir {
        IR::IO(io) => {
            let io_id = io.cache_key(&ctx).ok_or(EvaluationError::ExprEvalError(
                "Unable to generate cache key".to_string(),
            ))?;
            let value = ir.eval(ctx).await.map_err(|e| {
                EvaluationError::ExprEvalError(format!("Unable to evaluate: {}", e))
            })?;

            let value = value
                .into_json()
                .map_err(|e| EvaluationError::DeserializeError(e.to_string()))?;

            let owned_val = OwnedValue::from_string(value.to_string()).map_err(|e| {
                EvaluationError::DeserializeError(format!("Failed to deserialize IO value: {}", e))
            })?;
            Ok(IO::new(owned_val, io_id))
        }
        _ => Err(EvaluationError::ExprEvalError(
            "Unable to generate cache key".to_string(),
        )),
    }
}

#[cfg(test)]
pub mod tests {
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

        let store = Mutex::new(Store::new());

        let rt = crate::core::runtime::test::init(None);
        let request_ctx = RequestContext::new(rt);
        let gql_ctx = MockGraphqlContext { value: Default::default(), args: Default::default() };
        let ctx = EvaluationContext::new(&request_ctx, &gql_ctx);

        execute_ir(&plan, &store, &ctx).await.unwrap();
        let children = plan.as_children();
        let first = children.first().unwrap().to_owned();

        let store = store.into_inner();
        let synth = Synth::new(first, store);
        serde_json::to_string_pretty(&synth.synthesize(ctx)).unwrap()
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
