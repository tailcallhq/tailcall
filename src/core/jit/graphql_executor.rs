use std::future::Future;
use std::sync::Arc;

use async_graphql::{Data, Executor, Response};
use futures_util::stream::BoxStream;

use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::jit;
use crate::core::jit::ConstValueExecutor;

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

impl Executor for JITExecutor {
    fn execute(&self, request: async_graphql::Request) -> impl Future<Output = Response> + Send {
        let request = jit::Request::from(request);

        match ConstValueExecutor::new(&request, self.app_ctx.clone()) {
            Ok(exec) => {
                async {
                    let resp = exec.execute(&self.req_ctx, request).await;
                    resp.into_async_graphql()
                }
            }
            Err(_) => {
                todo!()
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
