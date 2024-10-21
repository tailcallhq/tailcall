use std::collections::BTreeMap;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_graphql::{Data, Executor, Response, Value};
use async_graphql_value::{ConstValue, Extensions};
use futures_util::stream::BoxStream;
use tailcall_hasher::TailcallHasher;

use crate::core::app_context::AppContext;
use crate::core::async_graphql_hyper::OperationId;
use crate::core::http::RequestContext;
use crate::core::jit::{self, ConstValueExecutor, OPHash};
use crate::core::lift::{CanLift, Lift};
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
    ) -> Response {
        let out = self
            .app_ctx
            .dedupe_operation_handler
            .dedupe(&self.operation_id, || {
                Box::pin(async move {
                    let resp = self.exec(exec, jit_request).await;
                    Ok(resp.lift())
                })
            })
            .await;

        out.map(|response| response.take()).unwrap_or_default()
    }

    #[inline(always)]
    fn req_hash(request: &async_graphql::Request) -> OPHash {
        let mut hasher = TailcallHasher::default();
        request.query.hash(&mut hasher);

        OPHash::new(hasher.finish())
    }
}

impl Clone for Lift<async_graphql::Response> {
    fn clone(&self) -> Self {
        let mut res = async_graphql::Response::new(self.data.clone())
            .cache_control(self.cache_control)
            .http_headers(self.http_headers.clone());
        res.errors = self.errors.clone();
        res.extensions = self.extensions.clone();
        res.into()
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

impl Executor for JITExecutor {
    fn execute(&self, request: async_graphql::Request) -> impl Future<Output = Response> + Send {
        let hash = Self::req_hash(&request);

        async move {
            let jit_request = jit::Request::from(request);
            let mut exec = if let Some(op) = self.app_ctx.operation_plans.get(&hash) {
                ConstValueExecutor::from(op.value().clone())
            } else {
                let exec = match ConstValueExecutor::try_new(&jit_request, &self.app_ctx) {
                    Ok(exec) => exec,
                    Err(error) => return Response::from_errors(vec![error.into()]),
                };
                self.app_ctx.operation_plans.insert(hash, exec.plan.clone());
                exec
            };

            if let Some(response) = std::mem::take(&mut exec.response) {
                response.into_async_graphql()
            } else if exec.plan.is_query() && exec.plan.is_dedupe {
                self.dedupe_and_exec(exec, jit_request).await
            } else {
                self.exec(exec, jit_request).await
            }
        }
    }

    fn execute_stream(
        &self,
        _: async_graphql::Request,
        _: Option<Arc<Data>>,
    ) -> BoxStream<'static, Response> {
        unimplemented!("streaming not supported")
    }
}
