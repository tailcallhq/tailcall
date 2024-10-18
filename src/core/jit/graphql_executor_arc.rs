use std::future::Future;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_graphql::{BatchRequest, Response};
use async_graphql_value::ConstValue;
use futures_util::stream::FuturesOrdered;
use futures_util::StreamExt;
use tailcall_hasher::TailcallHasher;

use crate::core::app_context::AppContext;
use crate::core::async_graphql_hyper::{CacheControl, OperationId};
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
    ) -> Arc<ByteResponse> {
        let is_introspection_query = self.app_ctx.blueprint.server.get_enable_introspection()
            && exec.plan.is_introspection_query;
        let jit_resp = exec
            .execute(&self.req_ctx, &jit_request)
            .await
            .into_async_graphql();
        let response = if is_introspection_query {
            let async_req = async_graphql::Request::from(jit_request).only_introspection();
            let async_resp = self.app_ctx.execute(async_req).await;
            jit_resp.merge_right(async_resp)
        } else {
            jit_resp
        };

        Arc::new(response.into())
    }

    #[inline(always)]
    async fn dedupe_and_exec(
        &self,
        exec: ConstValueExecutor,
        jit_request: jit::Request<ConstValue>,
    ) -> Arc<ByteResponse> {
        let out = self
            .app_ctx
            .dedupe_operation_handler_arc
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

impl JITArcExecutor {
    pub fn execute(
        &self,
        request: async_graphql::Request,
    ) -> impl Future<Output = Arc<ByteResponse>> + Send + '_ {
        let hash = Self::req_hash(&request);

        async move {
            let jit_request = jit::Request::from(request);
            let mut exec = if let Some(op) = self.app_ctx.operation_plans.get(&hash) {
                ConstValueExecutor::from(op.value().clone())
            } else {
                let exec = match ConstValueExecutor::try_new(&jit_request, &self.app_ctx) {
                    Ok(exec) => exec,
                    Err(error) => {
                        return Arc::new(Response::from_errors(vec![error.into()]).into())
                    }
                };
                self.app_ctx.operation_plans.insert(hash, exec.plan.clone());
                exec
            };

            if let Some(response) = std::mem::take(&mut exec.response) {
                Arc::new(response.into_async_graphql().into())
            } else if exec.plan.is_query() && exec.plan.dedupe {
                self.dedupe_and_exec(exec, jit_request).await
            } else {
                self.exec(exec, jit_request).await
            }
        }
    }

    /// Execute a GraphQL batch query.
    pub async fn execute_batch(&self, batch_request: BatchRequest) -> BatchResponse {
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

/// Represents a GraphQL response in a serialized byte format.
pub struct ByteResponse {
    /// The GraphQL response data serialized into a byte array.
    pub data: Vec<u8>,

    /// Information regarding cache policies for the response, such as max age
    /// and public/private settings.
    pub cache_control: CacheControl,

    /// Indicates whether graphql response contains error or not.
    pub is_ok: bool,
}

impl Default for ByteResponse {
    fn default() -> Self {
        async_graphql::Response::default().into()
    }
}

impl From<async_graphql::Response> for ByteResponse {
    fn from(response: async_graphql::Response) -> Self {
        ByteResponse {
            cache_control: CacheControl {
                max_age: response.cache_control.max_age,
                public: response.cache_control.public,
            },
            is_ok: response.errors.is_empty(),
            // Safely serialize the response to JSON bytes. Since the response is always valid, serialization is expected to succeed. 
            // In the unlikely event of a failure, default to an empty byte array.
            // TODO: return error instead of default value.
            data: serde_json::to_vec(&response).unwrap_or_default()
        }
    }
}

pub enum BatchResponse {
    Single(Arc<ByteResponse>),
    Batch(Vec<Arc<ByteResponse>>),
}

impl BatchResponse {
    pub fn is_ok(&self) -> bool {
        match self {
            BatchResponse::Single(s) => s.is_ok,
            BatchResponse::Batch(b) => b.iter().all(|s| s.is_ok),
        }
    }

    /// Modifies the cache control values with the provided one.
    pub fn cache_control(&self, cache_control: Option<&CacheControl>) -> CacheControl {
        match self {
            BatchResponse::Single(resp) => cache_control.unwrap_or(&resp.cache_control).clone(),
            BatchResponse::Batch(responses) => {
                responses.iter().fold(CacheControl::default(), |acc, resp| {
                    if let Some(cc) = cache_control {
                        acc.merge(cc)
                    } else {
                        acc.merge(&resp.cache_control)
                    }
                })
            }
        }
    }
}
