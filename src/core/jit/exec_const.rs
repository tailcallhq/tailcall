use std::sync::Arc;

use async_graphql_value::{ConstValue, Value};
use futures_util::future::join_all;

use super::context::Context;
use super::exec::{Executor, IRExecutor};
use super::{
    transform, BuildError, Error, OperationPlan, Pos, Positioned, Request, Response, Result,
};
use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::ir::model::IR;
use crate::core::ir::{self, EvalContext};
use crate::core::jit::synth::Synth;
use crate::core::jit::transform::InputResolver;
use crate::core::json::{JsonLike, JsonLikeList};
use crate::core::valid::Validator;
use crate::core::Transform;

/// A specialized executor that executes with async_graphql::Value
pub struct ConstValueExecutor {
    pub plan: OperationPlan<Value>,
    pub response: Option<Response<ConstValue>>,
}

impl From<OperationPlan<Value>> for ConstValueExecutor {
    fn from(plan: OperationPlan<Value>) -> Self {
        Self { plan, response: None }
    }
}

impl ConstValueExecutor {
    pub fn try_new(request: &Request<ConstValue>, app_ctx: &Arc<AppContext>) -> Result<Self> {
        let plan = request.create_plan(&app_ctx.blueprint)?;
        Ok(Self::from(plan))
    }

    pub async fn execute(
        mut self,
        req_ctx: &RequestContext,
        request: &Request<ConstValue>,
    ) -> Response<ConstValue> {
        let variables = &request.variables;
        let is_const = self.plan.is_const;

        // Attempt to skip unnecessary fields
        let Ok(plan) = transform::Skip::new(variables)
            .transform(self.plan)
            .to_result()
        else {
            // this shouldn't actually ever happen
            return Response::default()
                .with_errors(vec![Positioned::new(Error::Unknown, Pos::default())]);
        };

        // Attempt to replace variables in the plan with the actual values
        // TODO: operation from [ExecutableDocument] could contain definitions for
        // default values of arguments. That info should be passed to
        // [InputResolver] to resolve defaults properly
        let result = InputResolver::new(plan).resolve_input(variables);

        let plan = match result {
            Ok(plan) => plan,
            Err(err) => {
                return Response::default().with_errors(vec![Positioned::new(
                    BuildError::from(err).into(),
                    Pos::default(),
                )]);
            }
        };

        let exec = ConstValueExec::new(&plan, req_ctx);
        // PERF: remove this particular clone?
        let vars = request.variables.clone();
        let exe = Executor::new(&plan, exec);
        let store = exe.store().await;
        let synth = Synth::new(&plan, store, vars);
        let response = exe.execute(synth).await;

        // Cache the response if we know the output is always the same
        if is_const {
            self.response = Some(response.clone());
        }

        response
    }
}

struct ConstValueExec<'a> {
    plan: &'a OperationPlan<ConstValue>,
    req_context: &'a RequestContext,
}

impl<'a> ConstValueExec<'a> {
    pub fn new(plan: &'a OperationPlan<ConstValue>, req_context: &'a RequestContext) -> Self {
        Self { req_context, plan }
    }

    async fn call(
        &self,
        ctx: &'a Context<
            'a,
            <ConstValueExec<'a> as IRExecutor>::Input,
            <ConstValueExec<'a> as IRExecutor>::Output,
        >,
        ir: &'a IR,
    ) -> Result<<ConstValueExec<'a> as IRExecutor>::Output> {
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
                        tasks.push(async move {
                            let req_context = &self.req_context;
                            let mut eval_ctx = EvalContext::new(req_context, &ctx);
                            ir.eval(&mut eval_ctx).await
                        })
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
                        )))
                    } else {
                        Ok(Self::Output::default())
                    }
                })?)
            }
            _ => Ok(self.call(ctx, ir).await?),
        }
    }
}
