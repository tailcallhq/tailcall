use crate::core::ir::model::CacheKey;
use crate::core::jit::Eval;
use crate::core::jit::ir::IO;

pub fn eval_io(io: &IO, ctx: &Eval) -> Result<(), ()> {
    if ctx.app_ctx().blueprint.server.dedupe {

    }
    if let Some(key) = io.cache_key(ctx) {

    }else {

    }
}