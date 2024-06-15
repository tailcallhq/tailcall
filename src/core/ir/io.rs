use std::sync::Arc;

use async_graphql::from_value;
use async_graphql_value::ConstValue;
use reqwest::Request;

use super::{CacheKey, Eval, EvaluationContext, IoId, ResolverContextLike};
use crate::core::config::group_by::GroupBy;
use crate::core::config::GraphQLOperationType;
use crate::core::data_loader::{DataLoader, Loader};
use crate::core::graphql::{self, GraphqlDataLoader};
use crate::core::grpc::data_loader::GrpcDataLoader;
use crate::core::grpc::protobuf::ProtobufOperation;
use crate::core::grpc::request::execute_grpc_request;
use crate::core::grpc::request_template::RenderedRequestTemplate;
use crate::core::http::{
    cache_policy, DataLoaderRequest, HttpDataLoader, HttpFilter, RequestTemplate, Response,
};
use crate::core::ir::Error;
use crate::core::json::JsonLike;
use crate::core::valid::Validator;
use crate::core::worker::*;
use crate::core::{grpc, http, WorkerIO};

#[derive(Clone, Debug, strum_macros::Display)]
pub enum IO {
    Http {
        req_template: http::RequestTemplate,
        group_by: Option<GroupBy>,
        dl_id: Option<DataLoaderId>,
        http_filter: Option<HttpFilter>,
    },
    GraphQL {
        req_template: graphql::RequestTemplate,
        field_name: String,
        batch: bool,
        dl_id: Option<DataLoaderId>,
    },
    Grpc {
        req_template: grpc::RequestTemplate,
        group_by: Option<GroupBy>,
        dl_id: Option<DataLoaderId>,
    },
    Js {
        name: String,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct DataLoaderId(usize);
impl DataLoaderId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl Eval for IO {
    async fn eval<Ctx>(&self, ctx: &mut EvaluationContext<'_, Ctx>) -> Result<ConstValue, Error>
    where
        Ctx: ResolverContextLike + Sync,
    {
        // Note: Handled the case separately for performance reasons. It avoids cache
        // key generation when it's not required
        if !ctx.request_ctx.server.dedupe || !ctx.is_query() {
            return self.eval_inner(ctx).await;
        }
        if let Some(key) = self.cache_key(ctx) {
            ctx.request_ctx
                .cache
                .dedupe(&key, || async {
                    ctx.request_ctx
                        .dedupe_handler
                        .dedupe(&key, || self.eval_inner(ctx))
                        .await
                })
                .await
        } else {
            self.eval_inner(ctx).await
        }
    }
}

impl IO {
    async fn eval_inner<Ctx>(
        &self,
        ctx: &mut EvaluationContext<'_, Ctx>,
    ) -> Result<ConstValue, Error>
    where
        Ctx: ResolverContextLike + Sync,
    {
        match self {
            IO::Http { req_template, dl_id, http_filter, .. } => {
                let worker = &ctx.request_ctx.runtime.cmd_worker;
                let executor = HttpRequestExecutor::new(ctx, req_template, dl_id);
                let request = executor.init_request()?;
                let response = match (&worker, http_filter) {
                    (Some(worker), Some(http_filter)) => {
                        executor
                            .execute_with_worker(request, worker, http_filter)
                            .await?
                    }
                    _ => executor.execute(request).await?,
                };

                Ok(response.body)
            }
            IO::GraphQL { req_template, field_name, dl_id, .. } => {
                let req = req_template.to_request(ctx)?;

                let res = if ctx.request_ctx.upstream.batch.is_some()
                    && matches!(req_template.operation_type, GraphQLOperationType::Query)
                {
                    let data_loader: Option<&DataLoader<DataLoaderRequest, GraphqlDataLoader>> =
                        dl_id.and_then(|index| ctx.request_ctx.gql_data_loaders.get(index.0));
                    execute_request_with_dl(ctx, req, data_loader).await?
                } else {
                    execute_raw_request(ctx, req).await?
                };

                set_headers(ctx, &res);
                parse_graphql_response(ctx, res, field_name)
            }
            IO::Grpc { req_template, dl_id, .. } => {
                let rendered = req_template.render(ctx)?;

                let res = if ctx.request_ctx.upstream.batch.is_some() &&
                    // TODO: share check for operation_type for resolvers
                    matches!(req_template.operation_type, GraphQLOperationType::Query)
                {
                    let data_loader: Option<&DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>> =
                        dl_id.and_then(|index| ctx.request_ctx.grpc_data_loaders.get(index.0));
                    execute_grpc_request_with_dl(ctx, rendered, data_loader).await?
                } else {
                    let req = rendered.to_request()?;
                    execute_raw_grpc_request(ctx, req, &req_template.operation).await?
                };

                set_headers(ctx, &res);

                Ok(res.body)
            }
            IO::Js { name } => {
                if let Some((worker, value)) = ctx
                    .request_ctx
                    .runtime
                    .worker
                    .as_ref()
                    .zip(ctx.value().cloned())
                {
                    let val = worker.call(name, value).await?;
                    Ok(val.unwrap_or_default())
                } else {
                    Ok(ConstValue::Null)
                }
            }
        }
    }
}

impl<'a, Ctx: ResolverContextLike + Sync> CacheKey<EvaluationContext<'a, Ctx>> for IO {
    fn cache_key(&self, ctx: &EvaluationContext<'a, Ctx>) -> Option<IoId> {
        match self {
            IO::Http { req_template, .. } => req_template.cache_key(ctx),
            IO::Grpc { req_template, .. } => req_template.cache_key(ctx),
            IO::GraphQL { req_template, .. } => req_template.cache_key(ctx),
            IO::Js { .. } => None,
        }
    }
}

fn set_headers<Ctx: ResolverContextLike>(
    ctx: &EvaluationContext<'_, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    set_cache_control(ctx, res);
    set_cookie_headers(ctx, res);
    set_experimental_headers(ctx, res);
}

fn set_cache_control<Ctx: ResolverContextLike>(
    ctx: &EvaluationContext<'_, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    if ctx.request_ctx.server.get_enable_cache_control() && res.status.is_success() {
        if let Some(policy) = cache_policy(res) {
            ctx.request_ctx.set_cache_control(policy);
        }
    }
}

fn set_experimental_headers<Ctx: ResolverContextLike>(
    ctx: &EvaluationContext<'_, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    ctx.request_ctx.add_x_headers(&res.headers);
}

fn set_cookie_headers<Ctx: ResolverContextLike>(
    ctx: &EvaluationContext<'_, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    if res.status.is_success() {
        ctx.request_ctx.set_cookie_headers(&res.headers);
    }
}

async fn execute_raw_request<Ctx: ResolverContextLike>(
    ctx: &EvaluationContext<'_, Ctx>,
    req: Request,
) -> Result<Response<async_graphql::Value>, Error> {
    let response = ctx
        .request_ctx
        .runtime
        .http
        .execute(req)
        .await
        .map_err(Error::from)?
        .to_json()?;

    Ok(response)
}

async fn execute_raw_grpc_request<Ctx: ResolverContextLike>(
    ctx: &EvaluationContext<'_, Ctx>,
    req: Request,
    operation: &ProtobufOperation,
) -> Result<Response<async_graphql::Value>, Error> {
    execute_grpc_request(&ctx.request_ctx.runtime, operation, req)
        .await
        .map_err(Error::from)
}

async fn execute_grpc_request_with_dl<
    Ctx: ResolverContextLike,
    Dl: Loader<grpc::DataLoaderRequest, Value = Response<async_graphql::Value>, Error = Arc<Error>>,
>(
    ctx: &EvaluationContext<'_, Ctx>,
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

async fn execute_request_with_dl<
    'ctx,
    Ctx: ResolverContextLike,
    Dl: Loader<DataLoaderRequest, Value = Response<async_graphql::Value>, Error = Arc<Error>>,
>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    req: Request,
    data_loader: Option<&DataLoader<DataLoaderRequest, Dl>>,
) -> Result<Response<async_graphql::Value>, Error> {
    let headers = ctx
        .request_ctx
        .upstream
        .batch
        .clone()
        .map(|s| s.headers)
        .unwrap_or_default();
    let endpoint_key = crate::core::http::DataLoaderRequest::new(req, headers);

    Ok(data_loader
        .unwrap()
        .load_one(endpoint_key)
        .await
        .map_err(Error::from)?
        .unwrap_or_default())
}

fn parse_graphql_response<Ctx: ResolverContextLike>(
    ctx: &EvaluationContext<'_, Ctx>,
    res: Response<async_graphql::Value>,
    field_name: &str,
) -> Result<async_graphql::Value, Error> {
    let res: async_graphql::Response =
        from_value(res.body).map_err(|err| Error::DeserializeError(err.to_string()))?;

    for error in res.errors {
        ctx.add_error(error);
    }

    Ok(res
        .data
        .get_key(field_name)
        .map(|v| v.to_owned())
        .unwrap_or_default())
}

///
/// Executing a HTTP request is a bit more complex than just sending a request
/// and getting a response. There are optimizations and customizations that the
/// user might have configured. HttpRequestExecutor is responsible for handling
/// all of that.
struct HttpRequestExecutor<'a, 'ctx, Context: ResolverContextLike + Sync> {
    evaluation_ctx: &'ctx EvaluationContext<'a, Context>,
    data_loader: Option<&'a DataLoader<DataLoaderRequest, HttpDataLoader>>,
    request_template: &'a http::RequestTemplate,
}

impl<'a, 'ctx, Context: ResolverContextLike + Sync> HttpRequestExecutor<'a, 'ctx, Context> {
    pub fn new(
        evaluation_ctx: &'ctx EvaluationContext<'a, Context>,
        request_template: &'a RequestTemplate,
        id: &Option<DataLoaderId>,
    ) -> Self {
        let data_loader = if evaluation_ctx.request_ctx.is_batching_enabled() {
            id.and_then(|id| evaluation_ctx.request_ctx.http_data_loaders.get(id.0))
        } else {
            None
        };

        Self { evaluation_ctx, data_loader, request_template }
    }

    pub fn init_request(&self) -> Result<Request, Error> {
        Ok(self.request_template.to_request(self.evaluation_ctx)?)
    }

    async fn execute(&self, req: Request) -> Result<Response<async_graphql::Value>, Error> {
        let ctx = &self.evaluation_ctx;
        let is_get = req.method() == reqwest::Method::GET;
        let dl = &self.data_loader;
        let response = if is_get && dl.is_some() {
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
    async fn execute_with_worker(
        &self,
        mut request: reqwest::Request,
        worker: &Arc<dyn WorkerIO<Event, Command>>,
        http_filter: &HttpFilter,
    ) -> Result<Response<async_graphql::Value>, Error> {
        let js_request = WorkerRequest::try_from(&request)?;
        let event = Event::Request(js_request);

        let command = worker.call(&http_filter.on_request, event).await?;

        match command {
            Some(command) => match command {
                Command::Request(w_request) => {
                    let response = self.execute(w_request.into()).await?;
                    Ok(response)
                }
                Command::Response(w_response) => {
                    // Check if the response is a redirect
                    if (w_response.status() == 301 || w_response.status() == 302)
                        && w_response.headers().contains_key("location")
                    {
                        request
                            .url_mut()
                            .set_path(w_response.headers()["location"].as_str());
                        self.execute_with_worker(request, worker, http_filter).await
                    } else {
                        Ok(w_response.try_into()?)
                    }
                }
            },
            None => self.execute(request).await,
        }
    }
}
