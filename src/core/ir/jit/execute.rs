use serde_json_borrow::{OwnedValue, Value};
use tokio::sync::Mutex;

use crate::core::ir::jit::model::{Children, ExecutionPlan, Field};
use crate::core::ir::jit::store::{Data, Store, Stores};
use crate::core::ir::{
    CacheKey, Eval, EvaluationContext, EvaluationError, IoId, ResolverContextLike, IR,
};

struct IO {
    // drop this and use tuple instead
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
    store: &'a Mutex<Stores<IoId, OwnedValue>>,
    ctx: EvaluationContext<'a, Ctx>,
) -> Result<(), EvaluationError> {
    let mut stores = store.lock().await;
    for child in plan.as_children() {
        let mut call_ids = vec![];

        let mut store = Store::new();

        // TODO
        foo(&mut store, child, ctx.clone(), None, &mut call_ids).await?;
        println!("{:#?}", call_ids);
        let call_id = call_ids.first().unwrap();
        store.insert(IoId::new(0), Data { data: None });
        stores.insert(child.id.to_owned(), store);
    }
    Ok(())
}

// prolly we need IrId instead of IoId to avoid n+1
// prolly we need to change store such that we don't store list at all

#[async_recursion::async_recursion]
async fn foo<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    store: &mut Store<IoId, OwnedValue>,
    node: &'a Field<Children>,
    mut ctx: EvaluationContext<'a, Ctx>,
    parent_val: Option<&Value>,
    call_ids: &mut Vec<IoId>,
) -> Result<(), EvaluationError> {
    tracing::info!("Executing: {:?}", node.id);

    if let Some(ir) = node.ir.as_ref() {
        match parent_val {
            // TODO: maybe this should be kept in the if condition for IR
            Some(Value::Array(array)) => {
                for val in array {
                    // TODO: maybe collect call_id
                    foo(store, node, ctx.clone(), Some(val), call_ids).await?
                }
            }
            /*Some(Value::Object(obj)) => {
                tracing::info!("hx: {:?}",  obj);
                let val = obj.iter().find(|(k, _)| node.name.eq(*k)).map(|v| v.1);
                foo(store, node, ctx.clone(), val, counter, call_ids).await?
            }*/
            Some(val) => {
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
        call_ids.push(call_id.to_owned());

        for child in node.children() {
            if child.ir.is_some() {
                foo(
                    store,
                    child,
                    ctx.clone(),
                    Some(io.data.get_value()),
                    call_ids,
                )
                .await?;
            }
        }

        store.insert(call_id, Data { data: Some(io.data) });
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
            let ioid = io.cache_key(&ctx).ok_or(EvaluationError::ExprEvalError(
                "Unable to generate cache key".to_string(),
            ))?;
            let value = ir.eval(ctx).await.map_err(|e| {
                EvaluationError::ExprEvalError(format!("Unable to evaluate: {}", e))
            })?;

            let value = value
                .into_json()
                .map_err(|e| EvaluationError::DeserializeError(e.to_string()))?;

            // to_string might have issues as well, ideally we should directly convert to
            // OwnedValue
            let owned_val = OwnedValue::from_string(value.to_string()).map_err(|e| {
                EvaluationError::DeserializeError(format!("Failed to deserialize IO value: {}", e))
            })?;
            Ok(IO::new(owned_val, ioid))
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
    use crate::core::ir::jit::store::Stores;
    use crate::core::ir::jit::synth::Synth;
    use crate::core::ir::{EvaluationContext, IoId, ResolverContextLike};
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

        let stores = Mutex::new(Stores::new());

        let rt = crate::core::runtime::test::init(None);
        let request_ctx = RequestContext::new(rt);
        let gql_ctx = MockGraphqlContext { value: Default::default(), args: Default::default() };
        let ctx = EvaluationContext::new(&request_ctx, &gql_ctx);

        execute_ir(&plan, &stores, ctx.clone()).await.unwrap();
        let stores = stores.into_inner();
        // tracing::info!("{:#?}", store);
        let children = plan.as_children();
        let first = children.first().unwrap().to_owned();
        let store = stores.get(&first.id).unwrap();
        // tracing::info!("{:#?}", store);
        tracing::info!("{:?}", store.get(&IoId::new(101)));
        let synth = Synth::new(first, store.to_owned());
        serde_json::to_string_pretty(&synth.synthesize(&ctx)).unwrap()
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
