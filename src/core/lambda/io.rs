use core::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_graphql::from_value;
use async_graphql_value::ConstValue;
use reqwest::Request;

use super::{CacheKey, Eval, EvaluationContext, ResolverContextLike};
use crate::core::config::group_by::GroupBy;
use crate::core::config::GraphQLOperationType;
use crate::core::data_loader::{DataLoader, Loader};
use crate::core::graphql::{self, GraphqlDataLoader};
use crate::core::grpc::data_loader::GrpcDataLoader;
use crate::core::grpc::protobuf::ProtobufOperation;
use crate::core::grpc::request::execute_grpc_request;
use crate::core::grpc::request_template::RenderedRequestTemplate;
use crate::core::http::{cache_policy, DataLoaderRequest, HttpDataLoader, HttpFilter, Response};
use crate::core::json::JsonLike;
use crate::core::lambda::EvaluationError;
use crate::core::valid::Validator;
use crate::core::{grpc, http};

#[derive(Clone, Debug, strum_macros::Display)]
pub enum IO {
    Http {
        req_template: http::RequestTemplate,
        group_by: Option<GroupBy>,
        dl_id: Option<DataLoaderId>,
        http_filter: HttpFilter,
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
pub struct DataLoaderId(pub usize);

impl Eval for IO {
    fn eval<'a, Ctx: super::ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: super::EvaluationContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue, EvaluationError>> + 'a + Send>> {
        if ctx.request_ctx.upstream.dedupe {
            Box::pin(async move {
                let key = self.cache_key(&ctx);
                if let Some(key) = key {
                    ctx.request_ctx
                        .cache
                        .get_or_eval(key, move || Box::pin(self.eval_inner(ctx)))
                        .await
                        .as_ref()
                        .clone()
                } else {
                    self.eval_inner(ctx).await
                }
            })
        } else {
            Box::pin(self.eval_inner(ctx))
        }
    }
}

impl IO {
    fn eval_inner<'a, Ctx: super::ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: super::EvaluationContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue, EvaluationError>> + 'a + Send>> {
        Box::pin(async move {
            match self {
                IO::Http { req_template, dl_id, http_filter, .. } => {
                    let req = req_template.to_request(&ctx)?;
                    let is_get = req.method() == reqwest::Method::GET;

                    let res = if is_get && ctx.request_ctx.is_batching_enabled() {
                        let data_loader: Option<&DataLoader<DataLoaderRequest, HttpDataLoader>> =
                            dl_id.and_then(|index| ctx.request_ctx.http_data_loaders.get(index.0));
                        execute_request_with_dl(&ctx, req, data_loader, http_filter.clone()).await?
                    } else {
                        execute_raw_request(&ctx, req, http_filter.clone()).await?
                    };

                    if ctx.request_ctx.server.get_enable_http_validation() {
                        req_template
                            .endpoint
                            .output
                            .validate(&res.body)
                            .to_result()
                            .map_err(EvaluationError::from)?;
                    }

                    set_headers(&ctx, &res);

                    Ok(res.body)
                }
                IO::GraphQL { req_template, field_name, dl_id, .. } => {
                    let req = req_template.to_request(&ctx)?;

                    let res = if ctx.request_ctx.upstream.batch.is_some()
                        && matches!(req_template.operation_type, GraphQLOperationType::Query)
                    {
                        let data_loader: Option<&DataLoader<DataLoaderRequest, GraphqlDataLoader>> =
                            dl_id.and_then(|index| ctx.request_ctx.gql_data_loaders.get(index.0));
                        execute_request_with_dl(&ctx, req, data_loader, HttpFilter::default())
                            .await?
                    } else {
                        execute_raw_request(&ctx, req, HttpFilter::default()).await?
                    };

                    set_headers(&ctx, &res);
                    parse_graphql_response(&ctx, res, field_name)
                }
                IO::Grpc { req_template, dl_id, .. } => {
                    let rendered = req_template.render(&ctx)?;

                    let res = if ctx.request_ctx.upstream.batch.is_some() &&
                    // TODO: share check for operation_type for resolvers
                    matches!(req_template.operation_type, GraphQLOperationType::Query)
                    {
                        let data_loader: Option<
                            &DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>,
                        > = dl_id.and_then(|index| ctx.request_ctx.grpc_data_loaders.get(index.0));
                        execute_grpc_request_with_dl(&ctx, rendered, data_loader).await?
                    } else {
                        let req = rendered.to_request()?;
                        execute_raw_grpc_request(&ctx, req, &req_template.operation).await?
                    };

                    set_headers(&ctx, &res);

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
        })
    }
}

impl<'a, Ctx: ResolverContextLike<'a> + Sync + Send> CacheKey<EvaluationContext<'a, Ctx>> for IO {
    fn cache_key(&self, ctx: &EvaluationContext<'a, Ctx>) -> Option<u64> {
        match self {
            IO::Http { req_template, .. } => req_template.cache_key(ctx),
            IO::Grpc { req_template, .. } => req_template.cache_key(ctx),
            IO::GraphQL { req_template, .. } => req_template.cache_key(ctx),
            IO::Js { .. } => None,
        }
    }
}

fn set_headers<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    set_cache_control(ctx, res);
    set_cookie_headers(ctx, res);
    set_experimental_headers(ctx, res);
}

fn set_cache_control<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    if ctx.request_ctx.server.get_enable_cache_control() && res.status.is_success() {
        if let Some(policy) = cache_policy(res) {
            ctx.request_ctx.set_cache_control(policy);
        }
    }
}

fn set_experimental_headers<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    ctx.request_ctx.add_x_headers(&res.headers);
}

fn set_cookie_headers<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    if res.status.is_success() {
        ctx.request_ctx.set_cookie_headers(&res.headers);
    }
}

async fn execute_raw_request<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    req: Request,
    http_filter: HttpFilter,
) -> Result<Response<async_graphql::Value>, EvaluationError> {
    let response = ctx
        .request_ctx
        .runtime
        .http
        .execute_with(req, &http_filter)
        .await
        .map_err(EvaluationError::from)?
        .to_json()?;

    Ok(response)
}

async fn execute_raw_grpc_request<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    req: Request,
    operation: &ProtobufOperation,
) -> Result<Response<async_graphql::Value>, EvaluationError> {
    execute_grpc_request(&ctx.request_ctx.runtime, operation, req)
        .await
        .map_err(EvaluationError::from)
}

async fn execute_grpc_request_with_dl<
    'ctx,
    Ctx: ResolverContextLike<'ctx>,
    Dl: Loader<
        grpc::DataLoaderRequest,
        Value = Response<async_graphql::Value>,
        Error = Arc<anyhow::Error>,
    >,
>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    rendered: RenderedRequestTemplate,
    data_loader: Option<&DataLoader<grpc::DataLoaderRequest, Dl>>,
) -> Result<Response<async_graphql::Value>, EvaluationError> {
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
        .load_one(endpoint_key, HttpFilter::default())
        .await
        .map_err(EvaluationError::from)?
        .unwrap_or_default())
}

async fn execute_request_with_dl<
    'ctx,
    Ctx: ResolverContextLike<'ctx>,
    Dl: Loader<DataLoaderRequest, Value = Response<async_graphql::Value>, Error = Arc<anyhow::Error>>,
>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    req: Request,
    data_loader: Option<&DataLoader<DataLoaderRequest, Dl>>,
    http_filter: HttpFilter,
) -> Result<Response<async_graphql::Value>, EvaluationError> {
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
        .load_one(endpoint_key, http_filter)
        .await
        .map_err(EvaluationError::from)?
        .unwrap_or_default())
}

fn parse_graphql_response<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    res: Response<async_graphql::Value>,
    field_name: &str,
) -> Result<async_graphql::Value, EvaluationError> {
    let res: async_graphql::Response =
        from_value(res.body).map_err(|err| EvaluationError::DeserializeError(err.to_string()))?;

    for error in res.errors {
        ctx.add_error(error);
    }

    Ok(res
        .data
        .get_key(field_name)
        .map(|v| v.to_owned())
        .unwrap_or_default())
}
