use std::future::Future;
use std::sync::Arc;

use async_graphql::{Data, Executor, Response};
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

fn clone_request(req: &async_graphql::Request) -> async_graphql::Request {
    let mut request = async_graphql::Request::new(req.query.clone());
    request.variables = req.variables.clone();
    request.extensions = req.extensions.clone();
    request.operation_name = req.operation_name.clone();
    request.query = req.query.clone();
    request
}

impl Executor for JITExecutor {
    fn execute(&self, request: async_graphql::Request) -> impl Future<Output = Response> + Send {
        let jit_request = jit::Request::from(clone_request(&request));

        // execute only introspection requests with async graphql.
        let introspection_req = request.only_introspection();

        async {
            match ConstValueExecutor::new(&jit_request, self.app_ctx.clone()) {
                Ok(exec) => {
                    let is_introspection_query =
                        self.app_ctx.blueprint.server.get_enable_introspection()
                            && exec.plan.is_introspection_query;

                    let jit_resp = exec
                        .execute(&self.req_ctx, jit_request)
                        .await
                        .into_async_graphql();

                    if is_introspection_query {
                        let async_resp = self.app_ctx.execute(introspection_req).await;
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
