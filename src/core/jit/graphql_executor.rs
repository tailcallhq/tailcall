use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;

use async_graphql::{Data, Executor, Response, Value};
use async_graphql_value::Extensions;
use futures_util::stream::BoxStream;

use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::jit;
use crate::core::jit::ConstValueExecutor;
use crate::core::merge_right::MergeRight;

#[derive(Clone)]
pub struct JITExecutor {
    app_ctx: Arc<AppContext>,
    req_ctx: Arc<RequestContext>,
}

impl JITExecutor {
    pub fn new(app_ctx: Arc<AppContext>, req_ctx: Arc<RequestContext>) -> Self {
        Self { app_ctx, req_ctx }
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

        async {
            match ConstValueExecutor::new(&jit_request, self.app_ctx.clone()) {
                Ok(exec) => {
                    let is_introspection_query =
                        self.app_ctx.blueprint.server.get_enable_introspection()
                            && exec.plan.is_introspection_query;

                    let jit_resp = exec
                        .execute(&self.req_ctx, &jit_request)
                        .await
                        .into_async_graphql();

                    if is_introspection_query {
                        let async_req =
                            async_graphql::Request::from(jit_request).only_introspection();
                        let async_resp = self.app_ctx.execute(async_req).await;
                        jit_resp.merge_right(async_resp)
                    } else {
                        jit_resp
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
