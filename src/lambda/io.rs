use core::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use async_graphql_value::ConstValue;
use reqwest::Request;

use super::{Eval, EvaluationContext, ResolverContextLike};
use crate::config::group_by::GroupBy;
use crate::config::GraphQLOperationType;
use crate::data_loader::{DataLoader, Loader};
use crate::graphql::{self, GraphqlDataLoader};
use crate::grpc::data_loader::GrpcDataLoader;
use crate::grpc::protobuf::ProtobufOperation;
use crate::grpc::request::execute_grpc_request;
use crate::grpc::request_template::RenderedRequestTemplate;
use crate::http::{cache_policy, DataLoaderRequest, HttpDataLoader, Response};
use crate::json::JsonLike;
use crate::lambda::EvaluationError;
use crate::{grpc, http};

#[derive(Clone, Debug)]
pub enum IO {
    Http {
        req_template: http::RequestTemplate,
        group_by: Option<GroupBy>,
        dl_id: Option<DataLoaderId>,
    },
    GraphQLEndpoint {
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
}

#[derive(Clone, Copy, Debug)]
pub struct DataLoaderId(pub usize);

impl Eval for IO {
    fn eval<'a, Ctx: super::ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: &'a super::EvaluationContext<'a, Ctx>,
        _conc: &'a super::Concurrent,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
        Box::pin(async move {
            match self {
                IO::Http { req_template, dl_id, .. } => {
                    let req = req_template.to_request(ctx)?;
                    let is_get = req.method() == reqwest::Method::GET;

                    let res = if is_get && ctx.req_ctx.is_batching_enabled() {
                        let data_loader: Option<&DataLoader<DataLoaderRequest, HttpDataLoader>> =
                            dl_id.and_then(|index| ctx.req_ctx.http_data_loaders.get(index.0));
                        execute_request_with_dl(ctx, req, data_loader).await?
                    } else {
                        execute_raw_request(ctx, req).await?
                    };

                    if ctx.req_ctx.server.get_enable_http_validation() {
                        req_template
                            .endpoint
                            .output
                            .validate(&res.body)
                            .to_result()
                            .map_err(EvaluationError::from)?;
                    }

                    set_cache_control(ctx, &res);

                    Ok(res.body)
                }
                IO::GraphQLEndpoint { req_template, field_name, dl_id, .. } => {
                    let req = req_template.to_request(ctx)?;

                    let res = if ctx.req_ctx.upstream.batch.is_some()
                        && matches!(req_template.operation_type, GraphQLOperationType::Query)
                    {
                        let data_loader: Option<&DataLoader<DataLoaderRequest, GraphqlDataLoader>> =
                            dl_id.and_then(|index| ctx.req_ctx.gql_data_loaders.get(index.0));
                        execute_request_with_dl(ctx, req, data_loader).await?
                    } else {
                        execute_raw_request(ctx, req).await?
                    };

                    set_cache_control(ctx, &res);
                    parse_graphql_response(ctx, res, field_name)
                }
                IO::Grpc { req_template, dl_id, .. } => {
                    let rendered = req_template.render(ctx)?;

                    let res = if ctx.req_ctx.upstream.batch.is_some() &&
                    // TODO: share check for operation_type for resolvers
                    matches!(req_template.operation_type, GraphQLOperationType::Query)
                    {
                        let data_loader: Option<
                            &DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>,
                        > = dl_id.and_then(|index| ctx.req_ctx.grpc_data_loaders.get(index.0));
                        execute_grpc_request_with_dl(ctx, rendered, data_loader).await?
                    } else {
                        let req = rendered.to_request()?;
                        execute_raw_grpc_request(ctx, req, &req_template.operation).await?
                    };

                    set_cache_control(ctx, &res);

                    Ok(res.body)
                }
            }
        })
    }
}

fn set_cache_control<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    res: &Response<async_graphql::Value>,
) {
    if ctx.req_ctx.server.get_enable_cache_control() && res.status.is_success() {
        if let Some(policy) = cache_policy(res) {
            ctx.req_ctx.set_cache_control(policy);
        }
    }
}

async fn execute_raw_request<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    req: Request,
) -> Result<Response<async_graphql::Value>> {
    ctx.req_ctx
        .h_client
        .execute(req)
        .await
        .map_err(|e| EvaluationError::IOException(e.to_string()))?
        .to_json()
}

async fn execute_raw_grpc_request<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    req: Request,
    operation: &ProtobufOperation,
) -> Result<Response<async_graphql::Value>> {
    Ok(execute_grpc_request(&ctx.req_ctx.h2_client, operation, req)
        .await
        .map_err(|e| EvaluationError::IOException(e.to_string()))?)
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
) -> Result<Response<async_graphql::Value>> {
    let headers = ctx
        .req_ctx
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
        .map_err(|e| EvaluationError::IOException(e.to_string()))?
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
) -> Result<Response<async_graphql::Value>> {
    let headers = ctx
        .req_ctx
        .upstream
        .batch
        .clone()
        .map(|s| s.headers)
        .unwrap_or_default();
    let endpoint_key = crate::http::DataLoaderRequest::new(req, headers);

    Ok(data_loader
        .unwrap()
        .load_one(endpoint_key)
        .await
        .map_err(|e| EvaluationError::IOException(e.to_string()))?
        .unwrap_or_default())
}

fn parse_graphql_response<'ctx, Ctx: ResolverContextLike<'ctx>>(
    ctx: &EvaluationContext<'ctx, Ctx>,
    res: Response<async_graphql::Value>,
    field_name: &str,
) -> Result<async_graphql::Value> {
    let res: async_graphql::Response = serde_json::from_value(res.body.into_json()?)?;

    for error in res.errors {
        ctx.add_error(error);
    }

    Ok(res
        .data
        .get_key(field_name)
        .map(|v| v.to_owned())
        .unwrap_or_default())
}
