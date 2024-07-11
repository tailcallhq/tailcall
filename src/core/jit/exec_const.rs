use std::sync::Arc;

use async_graphql_value::ConstValue;

use super::context::Context;
use super::exec::{Executor, IRExecutor};
use super::synth::SynthConst;
use super::{Error, ExecutionPlan, Request, Response, Result};
use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::ir::model::IR;
use crate::core::ir::EvalContext;

/// A specialized executor that executes with async_graphql::Value
pub struct ConstValueExecutor {
    plan: ExecutionPlan<ConstValue>,
}

impl ConstValueExecutor {
    pub fn new(request: &Request<ConstValue>, app_ctx: Arc<AppContext>) -> Result<Self> {
        Ok(Self { plan: request.try_new(&app_ctx.blueprint)? })
    }

    pub async fn execute(
        self,
        req_ctx: &RequestContext,
        request: Request<ConstValue>,
    ) -> Response<ConstValue, Error> {
        let exec = ConstValueExec::new(req_ctx);
        let plan = self.plan;
        // TODO: drop the clones in plan
        let synth = SynthConst::new(plan.clone());
        let exe = Executor::new(plan, synth, exec);
        exe.execute(request).await
    }
}

struct ConstValueExec<'a> {
    req_context: &'a RequestContext,
}

impl<'a> ConstValueExec<'a> {
    pub fn new(ctx: &'a RequestContext) -> Self {
        Self { req_context: ctx }
    }
}

#[async_trait::async_trait]
impl<'ctx> IRExecutor for ConstValueExec<'ctx> {
    type Input = ConstValue;
    type Output = ConstValue;
    type Error = Error;

    async fn execute<'a>(
        &'a self,
        ir: &'a IR,
        ctx: &'a Context<'a, Self::Input, Self::Output>,
    ) -> Result<Self::Output> {
        let req_context = &self.req_context;
        let mut ctx = EvalContext::new(req_context, ctx);
        Ok(ir.eval(&mut ctx).await?)
    }
}
