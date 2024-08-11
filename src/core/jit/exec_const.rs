use std::sync::Arc;

use async_graphql_value::ConstValue;

use super::context::Context;
use super::exec::{Executor, IRExecutor, TypedValue};
use super::{Error, OperationPlan, Request, Response, Result};
use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::ir::model::IR;
use crate::core::ir::EvalContext;
use crate::core::jit::synth::Synth;

/// A specialized executor that executes with async_graphql::Value
pub struct ConstValueExecutor {
    plan: OperationPlan<ConstValue>,
}

impl ConstValueExecutor {
    pub fn new(request: &Request<ConstValue>, app_ctx: Arc<AppContext>) -> Result<Self> {
        Ok(Self { plan: request.create_plan(&app_ctx.blueprint)? })
    }

    pub async fn execute(
        self,
        req_ctx: &RequestContext,
        request: Request<ConstValue>,
    ) -> Response<ConstValue, Error> {
        let exec = ConstValueExec::new(req_ctx);
        let plan = self.plan;
        // TODO: drop the clones in plan
        let vars = request.variables.clone();
        let exe = Executor::new(plan.clone(), exec);
        let store = exe.store(request).await;
        let synth = Synth::new(plan, store, vars);
        exe.execute(synth).await
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

impl<'ctx> IRExecutor for ConstValueExec<'ctx> {
    type Input = ConstValue;
    type Output = ConstValue;
    type Error = Error;

    async fn execute<'a>(
        &'a self,
        ir: &'a IR,
        ctx: &'a Context<'a, Self::Input, Self::Output>,
    ) -> Result<TypedValue<Self::Output>> {
        let req_context = &self.req_context;
        let mut eval_ctx = EvalContext::new(req_context, ctx);

        Ok(ir
            .eval(&mut eval_ctx)
            .await
            .map(|value| TypedValue { value, type_name: eval_ctx.type_name.take() })?)
    }
}
