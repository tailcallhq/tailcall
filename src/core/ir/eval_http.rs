use std::sync::Arc;

use async_graphql::from_value;
use reqwest::Request;
use tailcall_valid::Validator;

use super::model::DataLoaderId;
use super::request::DynamicRequest;
use super::{EvalContext, ResolverContextLike};
use crate::core::data_loader::{DataLoader, Loader};
use crate::core::grpc::protobuf::ProtobufOperation;
use crate::core::grpc::request::execute_grpc_request;
use crate::core::grpc::request_template::RenderedRequestTemplate;
use crate::core::http::{
    cache_policy, DataLoaderRequest, HttpDataLoader, RequestTemplate, Response,
};
use crate::core::ir::Error;
use crate::core::json::JsonLike;
use crate::core::worker_hooks::WorkerHooks;
use crate::core::{grpc, http, worker, WorkerIO};

pub struct WorkerContext<'a> {
    pub worker: &'a Arc<dyn WorkerIO<worker::Event, worker::Command>>,
    pub js_worker:
        &'a Arc<dyn WorkerIO<async_graphql_value::ConstValue, async_graphql_value::ConstValue>>,
    pub js_hooks: &'a WorkerHooks,
}

impl<'a> WorkerContext<'a> {
    pub fn new(
        worker: &'a Arc<dyn WorkerIO<worker::Event, worker::Command>>,
        js_worker: &'a Arc<
            dyn WorkerIO<async_graphql_value::ConstValue, async_graphql_value::ConstValue>,
        >,
        js_hooks: &'a WorkerHooks,
    ) -> Self {
        Self { worker, js_worker, js_hooks }
    }
}

///
/// Executing a HTTP request is a bit more complex than just sending a request
/// and getting a response. There are optimizations and customizations that the
/// user might have configured. HttpRequestExecutor is responsible for handling
/// all of that.
pub struct EvalHttp<'a, 'ctx, Context: ResolverContextLike + Sync> {
    evaluation_ctx: &'ctx EvalContext<'a, Context>,
    data_loader: Option<&'a DataLoader<DataLoaderRequest, HttpDataLoader>>,
    request_template: &'a http::RequestTemplate,
}

impl<'a, 'ctx, Context: ResolverContextLike + Sync> EvalHttp<'a, 'ctx, Context> {
    pub fn new(
        evaluation_ctx: &'ctx EvalContext<'a, Context>,
        request_template: &'a RequestTemplate,
        id: &Option<DataLoaderId>,
    ) -> Self {
        let data_loader = if evaluation_ctx.request_ctx.is_batching_enabled() {
            id.and_then(|id| {
                evaluation_ctx
                    .request_ctx
                    .http_data_loaders
                    .get(id.as_usize())
            })
        } else {
            None
        };

        Self { evaluation_ctx, data_loader, request_template }
    }

    pub fn init_request(&self) -> Result<DynamicRequest<String>, Error> {
        let inner = self.request_template.to_request(self.evaluation_ctx)?;
        Ok(inner)
    }

    pub async fn execute(
        &self,
        req: DynamicRequest<String>,
    ) -> Result<Response<async_graphql::Value>, Error> {
        let ctx = &self.evaluation_ctx;
        let dl = &self.data_loader;
        let response = if dl.is_some() {
            execute_request_with_dl(ctx, req, self.data_loader).await?
        } else {
            execute_raw_request(ctx, req).await?
        };

        if ctx.request_ctx.server.get_enable_http_validation() {
            self.request_template
                .endpoint
                .output
                .validate(&response.body)
                .to_result()
                .map_err(Error::from)?;
        }

        set_headers(ctx, &response);

        Ok(response)
    }

    #[async_recursion::async_recursion]
    pub async fn execute_with_worker<'worker: 'async_recursion>(
        &self,
        mut request: DynamicRequest<String>,
        worker_ctx: WorkerContext<'worker>,
    ) -> Result<Response<async_graphql::Value>, Error> {
        // extract variables from the worker context.
        let js_hooks = worker_ctx.js_hooks;
        let worker = worker_ctx.worker;
        let js_worker = worker_ctx.js_worker;

        let response = match js_hooks.on_request(worker, request.request()).await? {
            Some(command) => match command {
                worker::Command::Request(w_request) => {
                    let response = self.execute(w_request.try_into()?).await?;
                    Ok(response)
                }
                worker::Command::Response(w_response) => {
                    // Check if the response is a redirect
                    if (w_response.status() == 301 || w_response.status() == 302)
                        && w_response.headers().contains_key("location")
                    {
                        request
                            .request_mut()
                            .url_mut()
                            .set_path(w_response.headers()["location"].as_str());
                        self.execute_with_worker(request, worker_ctx).await
                    } else {
                        Ok(w_response.try_into()?)
                    }
                }
            },
            None => self.execute(request).await,
        };

        // send the final response to JS script for futher evaluation.
        if let Ok(resp) = response {
            js_hooks.on_response(js_worker, resp).await
        } else {
            response
        }
    }
}

pub async fn execute_request_with_dl<
    'ctx,
    Ctx: ResolverContextLike,
    Dl: Loader<DataLoaderRequest, Value = Response<async_graphql::Value>, Error = Arc<anyhow::Error>>,
>(
    ctx: &EvalContext<'ctx, Ctx>,
    req: DynamicRequest<String>,
    data_loader: Option<&DataLoader<DataLoaderRequest, Dl>>,
) -> Result<Response<async_graphql::Value>, Error> {
    let headers = ctx
        .request_ctx
        .upstream
        .batch
        .clone()
        .map(|s| s.headers)
        .unwrap_or_default();

    let (req, batching_value) = req.into_parts();
    let endpoint_key =
        crate::core::http::DataLoaderRequest::new(req, headers).with_batching_value(batching_value);

    Ok(data_loader
        .unwrap()
        .load_one(endpoint_key)
        .await
        .map_err(Error::from)?
        .unwrap_or_default())
}

pub fn set_headers<Ctx: ResolverContextLike>(
    ctx: &EvalContext<'_, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    set_cache_control(ctx, res);
    set_cookie_headers(ctx, res);
    set_experimental_headers(ctx, res);
}

pub fn set_cache_control<Ctx: ResolverContextLike>(
    ctx: &EvalContext<'_, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    if ctx.request_ctx.server.get_enable_cache_control() && res.status.is_success() {
        if let Some(policy) = cache_policy(res) {
            ctx.request_ctx.set_cache_control(policy);
        }
    }
}

fn set_experimental_headers<Ctx: ResolverContextLike>(
    ctx: &EvalContext<'_, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    ctx.request_ctx.add_x_headers(&res.headers);
}

fn set_cookie_headers<Ctx: ResolverContextLike>(
    ctx: &EvalContext<'_, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    if res.status.is_success() {
        ctx.request_ctx.set_cookie_headers(&res.headers);
    }
}

pub async fn execute_raw_request<Ctx: ResolverContextLike>(
    ctx: &EvalContext<'_, Ctx>,
    req: DynamicRequest<String>,
) -> Result<Response<async_graphql::Value>, Error> {
    let response = ctx
        .request_ctx
        .runtime
        .http
        .execute(req.into_request())
        .await
        .map_err(Error::from)?
        .to_json()?;

    Ok(response)
}

pub async fn execute_raw_grpc_request<Ctx: ResolverContextLike>(
    ctx: &EvalContext<'_, Ctx>,
    req: Request,
    operation: &ProtobufOperation,
) -> Result<Response<async_graphql::Value>, Error> {
    execute_grpc_request(&ctx.request_ctx.runtime, operation, req)
        .await
        .map_err(Error::from)
}

pub async fn execute_grpc_request_with_dl<
    Ctx: ResolverContextLike,
    Dl: Loader<
        grpc::DataLoaderRequest,
        Value = Response<async_graphql::Value>,
        Error = Arc<anyhow::Error>,
    >,
>(
    ctx: &EvalContext<'_, Ctx>,
    rendered: RenderedRequestTemplate,
    data_loader: Option<&DataLoader<grpc::DataLoaderRequest, Dl>>,
) -> Result<Response<async_graphql::Value>, Error> {
    let headers = ctx
        .request_ctx
        .upstream
        .batch
        .clone()
        .map(|s| s.headers)
        .unwrap_or_default();
    let endpoint_key = grpc::DataLoaderRequest::new(rendered, headers);

    Ok(data_loader
        .unwrap()
        .load_one(endpoint_key)
        .await
        .map_err(Error::from)?
        .unwrap_or_default())
}

pub fn parse_graphql_response<Ctx: ResolverContextLike>(
    ctx: &EvalContext<'_, Ctx>,
    res: Response<async_graphql::Value>,
    field_name: &str,
) -> Result<async_graphql::Value, Error> {
    let res: async_graphql::Response =
        from_value(res.body).map_err(|err| Error::Deserialize(err.to_string()))?;

    for error in res.errors {
        ctx.add_error(error);
    }

    Ok(res
        .data
        .get_key(field_name)
        .map(|v| v.to_owned())
        .unwrap_or_default())
}
