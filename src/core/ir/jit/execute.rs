use std::collections::HashMap;

use serde_json_borrow::{OwnedValue, Value};
use tokio::sync::Mutex;

use crate::core::counter::{AtomicCounter, Count};
use crate::core::ir::jit::model::{Children, ExecutionPlan, Field, FieldId};
use crate::core::ir::jit::store::{Data, Store};
use crate::core::ir::{CallId, Eval, EvaluationContext, EvaluationError, ResolverContextLike, IR};

#[allow(unused)]
pub async fn execute_ir<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    plan: &'a ExecutionPlan,
    store: &'a Mutex<Store<CallId, OwnedValue>>,
    mut ctx: EvaluationContext<'a, Ctx>,
) -> Result<(), EvaluationError> {
    let counter = AtomicCounter::<usize>::default();
    let mut map = HashMap::new();
    if let Some(node) = plan.as_children().first() {
        execute_field(store, &mut ctx, None, &mut map, node, &counter, true).await?;
    }

    Ok(())
}

#[async_recursion::async_recursion]
async fn execute_field<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    store: &'a Mutex<Store<CallId, OwnedValue>>,
    ctx: &mut EvaluationContext<'a, Ctx>,
    parent: Option<&Value>,
    parent_extras: &mut HashMap<FieldId, CallId>,
    node: &'a Field<Children>,
    counter: &AtomicCounter<usize>,
    mut is_first: bool,
) -> Result<(), EvaluationError> {
    tracing::info!("Executing field: {}", node.name);
    match parent {
        Some(Value::Array(arr)) => {
            for val in arr {
                execute_field(
                    store,
                    ctx,
                    Some(val),
                    parent_extras,
                    node,
                    counter,
                    is_first,
                )
                .await?;
            }
        } // TODO: maybe handle object as well
        Some(val) => {
            let cv = serde_json::from_str(val.to_string().as_str()).map_err(|e| {
                EvaluationError::DeserializeError(format!("Failed to parse value: {}", e))
            })?;
            *ctx = ctx.with_value(cv);
        }
        _ => (),
    }
    if let Some(ir) = node.ir.as_ref() {
        let (value, call_id) = execute_io(counter, ctx.clone(), ir).await?;
        tracing::info!("{}", value.to_string());

        parent_extras.insert(node.id.to_owned(), call_id.to_owned()); // TODO: check if this is correct

        let mut extras = HashMap::new();
        if is_first {
            let mut extras = HashMap::new();
            extras.insert(node.id.to_owned(), call_id.to_owned());
            store
                .lock()
                .await
                .insert(CallId::new(0), Data { data: None, extras });
            is_first = false;
        }
        for child in node.children() {
            execute_field(
                store,
                ctx,
                Some(value.get_value()),
                &mut extras,
                child,
                counter,
                is_first,
            )
            .await?;
        }
        let data = Data { data: Some(value), extras };
        store.lock().await.insert(call_id, data);
    }
    Ok(())
}

async fn execute_io<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    counter: &AtomicCounter<usize>,
    ctx: EvaluationContext<'a, Ctx>,
    ir: &'a IR,
) -> Result<(OwnedValue, CallId), EvaluationError> {
    let call_id = counter.next();
    let value = ir
        .eval(ctx)
        .await
        .map_err(|e| EvaluationError::ExprEvalError(format!("Unable to evaluate: {}", e)))?;

    let str_value = value
        .into_json()
        .map_err(|e| EvaluationError::DeserializeError(e.to_string()))?;

    let str_value = str_value.to_string();

    let owned_value = OwnedValue::from_string(str_value)
        .map_err(|e| EvaluationError::DeserializeError(e.to_string()))?;

    Ok((owned_value, CallId::new(call_id)))
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
