use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;

use async_graphql::{Data, Executor, Response, ServerError, Value};
use async_graphql_value::{ConstValue, Extensions};
use futures_util::stream::BoxStream;

use crate::core::app_context::AppContext;
use crate::core::async_graphql_hyper::OperationId;
use crate::core::http::RequestContext;
use crate::core::jit;
use crate::core::jit::ConstValueExecutor;
use crate::core::merge_right::MergeRight;

#[derive(Clone)]
pub struct JITExecutor {
    app_ctx: Arc<AppContext>,
    req_ctx: Arc<RequestContext>,
    is_query: bool,
    operation_id: OperationId,
}

impl JITExecutor {
    pub fn new(
        app_ctx: Arc<AppContext>,
        req_ctx: Arc<RequestContext>,
        is_query: bool,
        operation_id: OperationId,
    ) -> Self {
        Self { app_ctx, req_ctx, is_query, operation_id }
    }
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
        let jit_request = jit::Request::from(request);

        async move {
            match ConstValueExecutor::new(&jit_request, &self.app_ctx) {
                Ok(exec) => {
                    if self.is_query && exec.plan.dedupe {
                        let out = self
                            .app_ctx
                            .dedupe_operation_handler
                            .dedupe(&self.operation_id, || {
                                Box::pin(async move {
                                    let resp = self.exec(exec, jit_request).await;
                                    Ok(Arc::new(resp))
                                })
                            })
                            .await;
                        let val = out.unwrap_or_default();
                        Arc::into_inner(val).unwrap_or_else(|| {
                            Response::from_errors(vec![ServerError::new(
                                "Deduplication failed",
                                None,
                            )])
                        })
                    } else {
                        self.exec(exec, jit_request).await
                    }
                }
                Err(error) => Response::from_errors(vec![error.into()]),
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
