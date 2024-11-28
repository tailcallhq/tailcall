use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use async_graphql_value::{ConstValue, Value};
use futures_util::future::join_all;
use tailcall_valid::Validator;

use super::context::Context;
use super::exec::{Executor, IRExecutor};
use super::graphql_error::GraphQLError;
use super::{
    transform, AnyResponse, BuildError, CompletedTasks, Error, Incremental, IncrementalItem,
    OperationPlan, Pending, Request, Response, Result,
};
use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::ir::model::IR;
use crate::core::ir::{self, EmptyResolverContext, EvalContext};
use crate::core::jit::synth::Synth;
use crate::core::jit::transform::InputResolver;
use crate::core::json::{JsonLike, JsonLikeList};
use crate::core::Transform;

use bytes::Bytes;
use futures::channel::mpsc;
use futures::SinkExt;
use tokio::sync::RwLock;

/// A specialized executor that executes with async_graphql::Value
pub struct ConstValueExecutor {
    pub plan: OperationPlan<Value>,
    pub tx: Arc<RwLock<Option<mpsc::Sender<anyhow::Result<Bytes>>>>>,
}

impl From<OperationPlan<Value>> for ConstValueExecutor {
    fn from(plan: OperationPlan<Value>) -> Self {
        Self { plan, tx: Arc::new(RwLock::new(None)) }
    }
}

impl ConstValueExecutor {
    pub fn try_new(request: &Request<ConstValue>, app_ctx: &Arc<AppContext>) -> Result<Self> {
        let plan = request.create_plan(&app_ctx.blueprint)?;
        Ok(Self::from(plan))
    }

    pub fn with_tx(self, tx: Arc<RwLock<Option<mpsc::Sender<anyhow::Result<Bytes>>>>>) -> Self {
        Self { plan: self.plan, tx: tx }
    }

    pub async fn execute<'a>(
        self,
        app_ctx: Arc<AppContext>,
        req_ctx: Arc<RequestContext>,
        request: Request<ConstValue>,
    ) -> AnyResponse<Vec<u8>> {
        // Run all the IRs in the before chain
        if let Some(ir) = &self.plan.before {
            let mut eval_context = EvalContext::new(&req_ctx, &EmptyResolverContext {});
            match ir.eval(&mut eval_context).await {
                Ok(_) => (),
                Err(err) => {
                    let resp: Response<ConstValue> = Response::default();
                    return resp
                        .with_errors(vec![GraphQLError::new(err.to_string(), None)])
                        .into();
                }
            }
        }

        let is_introspection_query =
            req_ctx.server.get_enable_introspection() && self.plan.is_introspection_query;
        let variables = &request.variables;

        // Attempt to skip unnecessary fields
        let Ok(plan) = transform::Skip::new(variables)
            .transform(self.plan)
            .to_result()
        else {
            let resp: Response<ConstValue> = Response::default();
            // this shouldn't actually ever happen
            return resp
                .with_errors(vec![GraphQLError::new(Error::Unknown.to_string(), None)])
                .into();
        };

        // Attempt to replace variables in the plan with the actual values
        // TODO: operation from [ExecutableDocument] could contain definitions for
        // default values of arguments. That info should be passed to
        // [InputResolver] to resolve defaults properly
        let result = InputResolver::new(plan).resolve_input(variables);

        let plan = match result {
            Ok(plan) => plan,
            Err(err) => {
                let resp: Response<ConstValue> = Response::default();
                return resp
                    .with_errors(vec![GraphQLError::new(
                        BuildError::from(err).to_string(),
                        None,
                    )])
                    .into();
            }
        };

        let vars = request.variables.clone();

        let exec = ConstValueExec::new(&plan, &req_ctx);
        // PERF: remove this particular clone?
        let exe = Executor::new(&plan, exec);
        let store = exe.store().await;
        let synth = Synth::new(&plan, store, vars.clone());

        let resp: Response<serde_json_borrow::Value> = exe.execute(&synth).await;

        // add `pending` and `has_next` to response.
        let resp = if !plan.deferred_fields.is_empty() {
            let mut pending_tasks = Vec::with_capacity(plan.deferred_fields.len());
            for field in plan.deferred_fields.iter() {
                if let Some(IR::Deferred { id, path, .. }) = &field.ir {
                    pending_tasks.push(Pending::new(id.as_u64(), id.to_string(), path.to_owned()));
                }
            }
            resp.pending(pending_tasks).has_next(Some(true))
        } else {
            resp
        };

        // add the pending to response.
        let response: AnyResponse<Vec<u8>> = if is_introspection_query {
            let async_req = async_graphql::Request::from(request.clone()).only_introspection();
            let async_resp = app_ctx.execute(async_req).await;

            resp.merge_with(&async_resp).into()
        } else {
            resp.into()
        };

        let bytes = response.to_bytes();
        let tx = self.tx.clone();
        {
            // Process base response first.
            let read_tx = tx.read().await;
            if let Some(sender) = &*read_tx {
                // Clone the sender so it can be used mutably outside the lock
                let mut sender = sender.clone();
                let _ = sender.send(Ok(bytes)).await.unwrap();
            }
        }

        // resposible for execution deferred fields.
        let tx = self.tx.clone();
        let total_deferred_fields = Arc::new(AtomicUsize::new(plan.deferred_fields.len()));

        let cloned_plan = plan.clone();
        let futures: Vec<_> = plan
            .deferred_fields
            .into_iter()
            .map(|field| {
                let tx = tx.clone();
                let is_introspection_query = is_introspection_query.clone();
                // let request = request.clone();
                // let app_ctx = app_ctx.clone();
                let vars = vars.clone();
                let cloned_plan = cloned_plan.clone();
                let cloned_req_ctx = req_ctx.clone();
                let total_deferred_fields_ = total_deferred_fields.clone();

                async move {
                    let mut deferred_plan = cloned_plan.clone();
                    deferred_plan.selection.clear();
                    deferred_plan.selection.push(field.clone());

                    let exec = ConstValueExec::new(&deferred_plan, &cloned_req_ctx);
                    let exe = Executor::new(&deferred_plan, exec);

                    let store = exe.store().await;
                    let synth = Synth::new(&deferred_plan, store, vars);

                    let resp: Response<serde_json_borrow::Value> = exe.execute(&synth).await;
                    let response: Incremental<serde_json_borrow::Value> = if is_introspection_query
                    {
                        // let async_req = async_graphql::Request::from(request).only_introspection();
                        // let async_resp = app_ctx.execute(async_req).await;
                        // resp.merge_with(&async_resp).into()
                        resp.into()
                    } else {
                        if let Some(IR::Deferred { id, .. }) = &field.ir {
                            let item = IncrementalItem::new(id.as_u64(), resp.data);
                            let completed = CompletedTasks::new(id.to_string());
                            Incremental::new(vec![item], vec![completed])
                        } else {
                            resp.into()
                        }
                    };
                    total_deferred_fields_.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    let response = response.has_next(
                        total_deferred_fields_.load(std::sync::atomic::Ordering::Relaxed) > 0,
                    );

                    let bytes = response.to_bytes();
                    let read_tx = tx.read().await;
                    if let Some(sender) = &*read_tx {
                        let mut sender = sender.clone();
                        let _ = sender.send(Ok(bytes)).await.unwrap();
                    }
                }
            })
            .collect();

        let _ = join_all(futures).await;

        AnyResponse::default()
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

impl IRExecutor for ConstValueExec<'_> {
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
