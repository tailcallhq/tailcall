use std::sync::Arc;

use async_graphql_value::{ConstValue, Value};
use bytes::Bytes;
use futures_util::future::join_all;
use hyper::body::Sender;
use serde_json_borrow::ObjectAsVec;
use tailcall_valid::Validator;
use tokio::sync::Mutex;

use super::context::Context;
use super::exec::{Executor, IRExecutor};
use super::graphql_error::GraphQLError;
use super::{transform, AnyResponse, BuildError, Error, OperationPlan, Request, Response, Result};
use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::ir::model::IR;
use crate::core::ir::{self, EmptyResolverContext, EvalContext};
use crate::core::jit::synth::Synth;
use crate::core::jit::transform::InputResolver;
use crate::core::json::{JsonLike, JsonLikeList};
use crate::core::Transform;

/// A specialized executor that executes with async_graphql::Value
pub struct ConstValueExecutor {
    pub plan: OperationPlan<Value>,
    pub sender: Option<Arc<Mutex<Sender>>>,
}

impl From<OperationPlan<Value>> for ConstValueExecutor {
    fn from(plan: OperationPlan<Value>) -> Self {
        Self { plan, sender: None }
    }
}

impl ConstValueExecutor {
    pub fn try_new(request: &Request<ConstValue>, app_ctx: &Arc<AppContext>) -> Result<Self> {
        let plan = request.create_plan(&app_ctx.blueprint)?;
        Ok(Self::from(plan))
    }

    pub fn with_sender(self, sender: Arc<Mutex<Sender>>) -> Self {
        Self { sender: Some(sender), ..self }
    }

    pub async fn execute<'a>(
        self,
        app_ctx: &Arc<AppContext>,
        req_ctx: &RequestContext,
        request: Request<ConstValue>,
    ) -> AnyResponse<Vec<u8>> {
        // Run all the IRs in the before chain
        if let Some(ir) = &self.plan.before {
            let mut eval_context = EvalContext::new(req_ctx, &EmptyResolverContext {});
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

        let exec = ConstValueExec::new(&plan, req_ctx);
        // PERF: remove this particular clone?
        let vars = request.variables.clone();
        let exe = Executor::new(&plan, exec);
        let store = exe.store().await;
        let synth = Synth::new(&plan, store, vars.clone());

        let resp: Response<serde_json_borrow::Value> = exe.execute(&synth).await;
        let resp = if plan.deferred_fields.is_empty() {
            resp
        } else {
            let mut pending_values = Vec::new();
            for field in plan.deferred_fields.iter() {
                let mut obj_vec = ObjectAsVec::default();
                if let Some(IR::Deferred { id, ir, path }) = &field.ir {
                    let s_path = path
                        .into_iter()
                        .map(|v| serde_json_borrow::Value::Str(std::borrow::Cow::Borrowed(v)))
                        .collect::<Vec<_>>();

                    obj_vec.insert(
                        "id",
                        serde_json_borrow::Value::Str(std::borrow::Cow::Owned(id.to_string())),
                    );
                    obj_vec.insert(
                        "label",
                        serde_json_borrow::Value::Str(std::borrow::Cow::Owned(id.to_string())),
                    );
                    obj_vec.insert("path", serde_json_borrow::Value::Array(s_path));
                }
                pending_values.push(serde_json_borrow::Value::Object(obj_vec));
            }

            if pending_values.len() > 0 {
                resp.has_next(Some(true))
                    .pending(Some(serde_json_borrow::Value::Array(pending_values)))
            } else {
                resp
            }
        };

        let response: AnyResponse<Vec<u8>> = if is_introspection_query {
            let async_req = async_graphql::Request::from(request.clone()).only_introspection();
            let async_resp = app_ctx.execute(async_req).await;

            resp.merge_with(&async_resp).into()
        } else {
            resp.into()
        };

        {
            let local_sender = self.sender.clone().unwrap();
            let mut sender = local_sender.lock().await;
            let bytes = Bytes::from(response.body.to_vec());
            let _ = sender.send_data(bytes.clone()).await.unwrap();
        }

        // resposible for execution deferred fields.
        for field in plan.deferred_fields.iter() {
            let mut deferred_plan = plan.clone();
            deferred_plan.selection.clear();
            deferred_plan.selection.push(field.clone());

            let exec = ConstValueExec::new(&deferred_plan, req_ctx);
            let exe = Executor::new(&deferred_plan, exec);
            let store = exe.store().await;
            let synth = Synth::new(&deferred_plan, store, vars.clone());

            let resp: Response<serde_json_borrow::Value> = exe.execute(&synth).await;
            let response: AnyResponse<Vec<u8>> = if is_introspection_query {
                let async_req = async_graphql::Request::from(request.clone()).only_introspection();
                let async_resp = app_ctx.execute(async_req).await;
                resp.merge_with(&async_resp).into()
            } else {
                resp.into()
            };

            let sender = self.sender.clone();

            // let data = r#"{ "incremental":[ { "id": "0", "data": { "user": { "name": "Tatooine" } } } ], "completed": [{"id": "0"}], "hasNext": false }"#;
            // let response:serde_json_borrow::Value = serde_json::from_str(data).unwrap();
            // let bytes = serde_json::to_vec(&response).unwrap();
            tokio::spawn(async move {
                let mut sender = sender.clone().unwrap();
                let mut sender = sender.lock().await;
                let bytes = Bytes::from(response.body.to_vec());
                let result = sender.send_data(bytes).await;
            });

            // println!("[Finder]: after {:#?}", result);
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
