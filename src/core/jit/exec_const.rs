use std::sync::Arc;

use async_graphql_value::ConstValue;
use futures_util::future::join_all;

use super::context::Context;
use super::exec::{Executor, IRExecutor};
use super::{Error, OperationPlan, Request, Response, Result};
use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::ir::model::IR;
use crate::core::ir::{self, EvalContext};
use crate::core::jit::synth::Synth;
use crate::core::json::{JsonLike, JsonLikeList};

/// A specialized executor that executes with async_graphql::Value
pub struct ConstValueExecutor {
    pub plan: OperationPlan<ConstValue>,
}

impl ConstValueExecutor {
    pub fn new(request: &Request<ConstValue>, app_ctx: &Arc<AppContext>) -> Result<Self> {
        Ok(Self { plan: request.create_plan(&app_ctx.blueprint)? })
    }

    pub async fn execute(
        self,
        req_ctx: &RequestContext,
        request: &Request<ConstValue>,
    ) -> Response<ConstValue, Error> {
        let plan = self.plan;
        // TODO: drop the clones in plan
        let exec = ConstValueExec::new(plan.clone(), req_ctx);
        let vars = request.variables.clone();
        let exe = Executor::new(plan.clone(), exec);
        let store = exe.store().await;
        let synth = Synth::new(plan, store, vars);
        exe.execute(synth).await
    }
}

struct ConstValueExec<'a> {
    plan: OperationPlan<ConstValue>,
    req_context: &'a RequestContext,
}

impl<'a> ConstValueExec<'a> {
    pub fn new(plan: OperationPlan<ConstValue>, req_context: &'a RequestContext) -> Self {
        Self { req_context, plan }
    }

    async fn call(
        &self,
        ctx: &'a Context<'a, <Self as IRExecutor>::Input, <Self as IRExecutor>::Output>,
        ir: &'a IR,
    ) -> Result<<Self as IRExecutor>::Output>
    where
        <Self as IRExecutor>::Input: JsonLike<'a>,
        <Self as IRExecutor>::Output: JsonLike<'a>,
    {
        // if parent value is null do not try to resolve child fields
        if matches!(ctx.value(), Some(v) if v.is_null()) {
            return Ok(Default::default());
        }

        let req_context = &self.req_context;
        let mut eval_ctx = EvalContext::new(req_context, ctx);

        Ok(ir.eval(&mut eval_ctx).await?)
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
    ) -> Result<Self::Output> {
        let field = ctx.field();

        match ctx.value() {
            // TODO: check that field is expected list and it's a list of the required deepness
            Some(value) if value.as_array().is_some() => {
                let mut tasks = Vec::new();

                // collect the async tasks first before creating the final result
                value.for_each(&mut |value| {
                    // execute the resolver only for fields that are related to current value
                    // for fragments on union/interface
                    if self.plan.field_is_part_of_value(field, value) {
                        let ctx = ctx.with_value(value);
                        tasks.push(async move { self.call(&ctx, ir).await })
                    }
                });

                let results = join_all(tasks).await;

                let mut iter = results.into_iter();

                // map input value to the calculated results preserving the shape
                // of the input
                Ok(value.map_ref(&mut |value| {
                    // for fragments on union/interface we will
                    // have less entries for resolved values based on the type
                    // pull from the result only field is related and fill with null otherwise
                    if self.plan.field_is_part_of_value(field, value) {
                        iter.next().unwrap_or(Err(ir::Error::IO(
                            "Expected value to be present".to_string(),
                        )
                        .into()))
                    } else {
                        Ok(Self::Output::default())
                    }
                })?)
            }
            _ => Ok(self.call(ctx, ir).await?),
        }
    }
}
