use std::collections::BTreeMap;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_graphql::{BatchRequest, Response, Value};
use async_graphql_value::{ConstValue, Extensions};
use futures_util::stream::FuturesOrdered;
use futures_util::StreamExt;
use tailcall_hasher::TailcallHasher;

use super::{AnyResponse, BatchResponse};
use crate::core::app_context::AppContext;
use crate::core::async_graphql_hyper::OperationId;
use crate::core::http::RequestContext;
use crate::core::jit::{self, ConstValueExecutor, OPHash};
use crate::core::merge_right::MergeRight;

#[derive(Clone)]
pub struct JITExecutor {
    app_ctx: Arc<AppContext>,
    req_ctx: Arc<RequestContext>,
    operation_id: OperationId,
}

impl JITExecutor {
    pub fn new(
        app_ctx: Arc<AppContext>,
        req_ctx: Arc<RequestContext>,
        operation_id: OperationId,
    ) -> Self {
        Self { app_ctx, req_ctx, operation_id }
    }

    #[inline(always)]
    async fn exec(
        &self,
        exec: ConstValueExecutor,
        jit_request: jit::Request<ConstValue>,
    ) -> AnyResponse<Vec<u8>> {
        let is_introspection_query = self.app_ctx.blueprint.server.get_enable_introspection()
            && exec.plan.is_introspection_query;
        let response = exec
            .execute(&self.req_ctx, &jit_request)
            .await;
        // let response = if is_introspection_query {
        //     let async_req = async_graphql::Request::from(jit_request).only_introspection();
        //     let async_resp = self.app_ctx.execute(async_req).await;
        //     response.merge_right(async_resp)
        // } else {
        //     response
        // };

        response.into()
    }

    #[inline(always)]
    async fn dedupe_and_exec(
        &self,
        exec: ConstValueExecutor,
        jit_request: jit::Request<ConstValue>,
    ) -> AnyResponse<Vec<u8>> {
        let out = self
            .app_ctx
            .dedupe_operation_handler
            .dedupe(&self.operation_id, || {
                Box::pin(async move {
                    let resp = self.exec(exec, jit_request).await;
                    Ok(resp)
                })
            })
            .await;

        out.unwrap_or_default()
    }

    #[inline(always)]
    fn req_hash(request: &async_graphql::Request) -> OPHash {
        let mut hasher = TailcallHasher::default();
        request.query.hash(&mut hasher);

        OPHash::new(hasher.finish())
    }
}

impl JITExecutor {
    pub fn execute(
        &self,
        request: async_graphql::Request,
    ) -> impl Future<Output = AnyResponse<Vec<u8>>> + Send + '_ {
        let hash = Self::req_hash(&request);

        async move {
            let jit_request = jit::Request::from(request);
            let mut exec = if let Some(op) = self.app_ctx.operation_plans.get(&hash) {
                ConstValueExecutor::from(op.value().clone())
            } else {
                let exec = match ConstValueExecutor::try_new(&jit_request, &self.app_ctx) {
                    Ok(exec) => exec,
                    Err(error) => return Response::from_errors(vec![error.into()]).into(),
                };
                self.app_ctx.operation_plans.insert(hash, exec.plan.clone());
                exec
            };

            if let Some(response) = std::mem::take(&mut exec.response) {
                response.into_async_graphql().into()
            } else if exec.plan.is_query() && exec.plan.is_dedupe {
                self.dedupe_and_exec(exec, jit_request).await
            } else {
                self.exec(exec, jit_request).await
            }
        }
    }

    /// Execute a GraphQL batch query.
    pub async fn execute_batch(&self, batch_request: BatchRequest) -> BatchResponse<Vec<u8>> {
        match batch_request {
            BatchRequest::Single(request) => BatchResponse::Single(self.execute(request).await),
            BatchRequest::Batch(requests) => {
                let futs = FuturesOrdered::from_iter(
                    requests.into_iter().map(|request| self.execute(request)),
                );
                let responses = futs.collect::<Vec<_>>().await;
                BatchResponse::Batch(responses)
            }
        }
    }
}

impl From<jit::Request<Value>> for async_graphql::Request {
    fn from(value: jit::Request<Value>) -> Self {
        let mut request = async_graphql::Request::new(value.query);
        request.variables.extend(
            value
                .variables
                .into_hashmap()
                .into_iter()
                .map(|(k, v)| (async_graphql::Name::new(k), v))
                .collect::<BTreeMap<_, _>>(),
        );
        request.extensions = Extensions(value.extensions);
        request.operation_name = value.operation_name;
        request
    }
}
