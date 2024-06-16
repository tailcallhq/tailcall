use async_graphql_value::ConstValue;

use super::http_executor::{
    execute_grpc_request_with_dl, execute_raw_grpc_request, execute_raw_request,
    execute_request_with_dl, parse_graphql_response, set_headers, HttpRequestExecutor,
};
use super::{CacheKey, Eval, EvaluationContext, IoId, ResolverContextLike};
use crate::core::config::group_by::GroupBy;
use crate::core::config::GraphQLOperationType;
use crate::core::data_loader::DataLoader;
use crate::core::graphql::{self, GraphqlDataLoader};
use crate::core::grpc::data_loader::GrpcDataLoader;
use crate::core::http::{DataLoaderRequest, HttpFilter};
use crate::core::ir::EvaluationError;
use crate::core::{grpc, http};

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
    async fn eval<Ctx>(
        &self,
        ctx: &mut EvaluationContext<'_, Ctx>,
    ) -> Result<ConstValue, EvaluationError>
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
    ) -> Result<ConstValue, EvaluationError>
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
