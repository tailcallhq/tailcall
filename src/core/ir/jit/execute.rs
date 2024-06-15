use serde_json_borrow::{OwnedValue, Value};
use tokio::sync::Mutex;

use crate::core::ir::jit::model::{Children, ExecutionPlan, Field};
use crate::core::ir::jit::store::Store;
use crate::core::ir::{Eval, EvaluationContext, EvaluationError, IoId, ResolverContextLike, IR};

pub struct IOExit {
    pub data: OwnedValue,
    pub id: Option<IoId>,
}

impl IOExit {
    pub fn new(value: OwnedValue, id: Option<IoId>) -> Self {
        Self { data: value, id }
    }
}

pub(super) async fn execute_ir<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
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
        if let Some(call_id) = call_id {
            store.insert(call_id, io.data);
        }
    }

    Ok(())
}

async fn execute<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    ir: &'a IR,
    ctx: EvaluationContext<'a, Ctx>,
) -> Result<IOExit, EvaluationError> {
    // TODO: should implement some kind of key for all fields of IR
    ir.eval(ctx).await
}
