use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use serde_json_borrow::OwnedValue;

use crate::core::ir::{
    CacheKey, Eval, EvaluationContext, EvaluationError, IoId, IR, ResolverContextLike,
};
use crate::core::ir::jit::model::{Children, ExecutionPlan, Field, FieldId};
use crate::core::ir::jit::store::{Data, Store};

pub async fn execute_ir<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    plan: &'a ExecutionPlan,
    store: &'a Mutex<Store<IoId, OwnedValue>>,
    mut ctx: EvaluationContext<'a, Ctx>,
) -> Result<(), EvaluationError> {

    let mut ids = HashMap::new();
    let mut is_first = true;
    let mut store_lock = store.lock().unwrap();
    for parent in plan.parents().iter() {
        if let Some(ir) = parent.ir.as_ref() {
            let (value, io_id) = execute_field(&mut ctx, ir).await?;
            if is_first {
                let mut extras = HashMap::new();
                extras.insert(parent.id.to_owned(), io_id.to_owned());
                store_lock.insert(IoId::new(0), Data { data: None, extras });
                is_first = false;
            }
            let data = Data { data: Some(value), extras: HashMap::new() };
            ids.insert(parent.id.to_owned(), io_id.to_owned());
            store_lock.insert(io_id, data);
        }
    }
    drop(store_lock);
    for ch in plan.children().iter() {
        resolve_extras(store, ch, &ids);
    }
    Ok(())
}

fn resolve_extras(
    store: &Mutex<Store<IoId, OwnedValue>>,
    child: &Field<Children>,
    helper: &HashMap<FieldId, IoId>,
) {
    let mut store_lock = store.lock().unwrap();
    if let Some(io_id) = helper.get(&child.id) {
        let data = store_lock.get_mut(io_id).unwrap();
        for child in child.children() {
            if child.ir.as_ref().is_some() {
                data.extras.insert(
                    child.id.to_owned(),
                    helper.get(&child.id).unwrap().to_owned(),
                );
            }
        }
    }
}

async fn execute_field<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    ctx: &mut EvaluationContext<'a, Ctx>,
    ir: &'a IR,
) -> Result<(OwnedValue, IoId), EvaluationError> {
    let val = ir.eval(ctx.clone()).await.map_err(|e| {
        EvaluationError::ExprEvalError(format!("Unable to evaluate: {}", e))
    })?;

    if let Some((value, io_id)) = val {
        *ctx = ctx.with_value(value.clone());
        let str_value = value
            .into_json()
            .map_err(|e| EvaluationError::DeserializeError(e.to_string()))?;
        let str_value = str_value.to_string();
        let owned_value = OwnedValue::from_string(str_value)
            .map_err(|e| EvaluationError::DeserializeError(e.to_string()))?;
        return Ok((owned_value, io_id));
    }
    Err(EvaluationError::ExprEvalError(
        "Unable to evaluate".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_graphql::{SelectionField, Value};
    use async_graphql_value::Name;
    use indexmap::IndexMap;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::http::RequestContext;
    use crate::core::ir::EvaluationContext;
    use crate::core::ir::jit::builder::ExecutionPlanBuilder;
    use crate::core::ir::jit::execute::execute_ir;
    use crate::core::ir::jit::store::Store;
    use crate::core::ir::jit::synth::Synth;
    use crate::core::ResolverContextLike;
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
        let store = store.into_inner().unwrap();
        let children = plan.children();
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
