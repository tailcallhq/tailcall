use std::future::Future;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_graphql::{BatchRequest, Response};
use async_graphql_value::ConstValue;
use futures_util::stream::FuturesOrdered;
use tailcall_hasher::TailcallHasher;
use futures_util::StreamExt;

use crate::core::app_context::AppContext;
use crate::core::async_graphql_hyper::OperationId;
use crate::core::http::RequestContext;
use crate::core::jit::{self, ConstValueExecutor, OPHash};
use crate::core::merge_right::MergeRight;

#[derive(Clone)]
pub struct JITArcExecutor {
    app_ctx: Arc<AppContext>,
    req_ctx: Arc<RequestContext>,
    operation_id: OperationId,
}

impl JITArcExecutor {
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
    ) -> Response {
        let is_introspection_query = self.app_ctx.blueprint.server.get_enable_introspection()
            && exec.plan.is_introspection_query;
        let jit_resp = exec
            .execute(&self.req_ctx, &jit_request)
            .await
            .into_async_graphql();
        if is_introspection_query {
            let async_req = async_graphql::Request::from(jit_request).only_introspection();
            let async_resp = self.app_ctx.execute(async_req).await;
            jit_resp.merge_right(async_resp)
        } else {
            jit_resp
        }
    }

    #[inline(always)]
    async fn dedupe_and_exec(
        &self,
        exec: ConstValueExecutor,
        jit_request: jit::Request<ConstValue>,
    ) -> Arc<Response> {
        let out = self
            .app_ctx
            .dedupe_operation_handler_arc
            .dedupe(&self.operation_id, || {
                Box::pin(async move {
                    let resp = self.exec(exec, jit_request).await;
                    Ok(Arc::new(resp))
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

impl JITArcExecutor {
    pub fn execute(
        &self,
        request: async_graphql::Request,
    ) -> impl Future<Output = Arc<Response>> + Send + '_ {
        let hash = Self::req_hash(&request);

        async move {
            let jit_request = jit::Request::from(request);
            let mut exec = if let Some(op) = self.app_ctx.operation_plans.get(&hash) {
                ConstValueExecutor::from(op.value().clone())
            } else {
                let exec = match ConstValueExecutor::try_new(&jit_request, &self.app_ctx) {
                    Ok(exec) => exec,
                    Err(error) => return Arc::new(Response::from_errors(vec![error.into()])),
                };
                self.app_ctx.operation_plans.insert(hash, exec.plan.clone());
                exec
            };

            let resp = if let Some(response) = std::mem::take(&mut exec.response) {
                Arc::new(response.into_async_graphql())
            } else if exec.plan.is_query() && exec.plan.dedupe {
                self.dedupe_and_exec(exec, jit_request).await
            } else {
                Arc::new(self.exec(exec, jit_request).await)
            };
            resp
        }
    }

    /// Execute a GraphQL batch query.
    pub fn execute_batch(
        &self,
        batch_request: BatchRequest,
    ) -> impl Future<Output = BatchResponse> + Send + '_ {
        async {
            match batch_request {
                BatchRequest::Single(request) => BatchResponse::Single(self.execute(request).await),
                BatchRequest::Batch(requests) => {
                    let futs = FuturesOrdered::from_iter(
                        requests.into_iter().map(|request| self.execute(request)),
                    );
                    let ans = futs.collect::<Vec<_>>().await;
                    BatchResponse::Batch(ans)
                }
            }
        }
    }
}


pub enum BatchResponse {
    Single(Arc<Response>),
    Batch(Vec<Arc<Response>>)
}