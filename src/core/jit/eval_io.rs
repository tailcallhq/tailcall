use serde_json_borrow::{OwnedValue, Value};
use crate::core::ir::Error;
use crate::core::ir::model::CacheKey;
use crate::core::jit::Eval;
use crate::core::jit::ir::IO;

pub async fn eval_io<'a>(io: &'a IO, ctx: &'a Eval<'a>) -> Result<Value<'a>, Error> {
    if ctx.app_ctx().blueprint.server.dedupe {
        // TODO: CHECK IF IT'S A QUERY
        return eval_io_inner(io, ctx).await;
    }
    if let Some(key) = io.cache_key(ctx) {
        let ans = ctx.req_ctx().cache.dedupe(&key, || async {
            ctx.req_ctx().dedupe_handler.dedupe(&key, || eval_io_inner(io, ctx)).await
        }).await?;
        Ok(ans)
    } else {
        eval_io_inner(io, ctx).await
    }
}

async fn eval_io_inner<'a>(io: &'a IO, ctx: &'a Eval<'a>) -> Result<Value<'a>, Error> {
    todo!()
}
