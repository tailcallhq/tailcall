use std::collections::{BTreeSet, HashSet};

use async_graphql_value::ConstValue;

use super::eval_http::{
    execute_grpc_request_with_dl, execute_raw_grpc_request, execute_raw_request,
    execute_request_with_dl, parse_graphql_response, set_headers, EvalHttp,
};
use super::model::{CacheKey, IO};
use super::{EvalContext, ResolverContextLike};
use crate::core::data_loader::DataLoader;
use crate::core::graphql::GraphqlDataLoader;
use crate::core::grpc;
use crate::core::grpc::data_loader::GrpcDataLoader;
use crate::core::http::DataLoaderRequest;
use crate::core::ir::Error;
use crate::core::{config::GraphQLOperationType, data_loader::Loader, json::JsonLike};

pub async fn eval_io<Ctx>(io: &IO, ctx: &mut EvalContext<'_, Ctx>) -> Result<ConstValue, Error>
where
    Ctx: ResolverContextLike + Sync,
{
    // Note: Handled the case separately for performance reasons. It avoids cache
    // key generation when it's not required
    if !ctx.request_ctx.server.dedupe || !ctx.is_query() {
        return eval_io_inner(io, ctx).await;
    }
    if let Some(key) = io.cache_key(ctx) {
        ctx.request_ctx
            .cache
            .dedupe(&key, || async {
                ctx.request_ctx
                    .dedupe_handler
                    .dedupe(&key, || eval_io_inner(io, ctx))
                    .await
            })
            .await
    } else {
        eval_io_inner(io, ctx).await
    }
}

async fn eval_io_inner<Ctx>(io: &IO, ctx: &mut EvalContext<'_, Ctx>) -> Result<ConstValue, Error>
where
    Ctx: ResolverContextLike + Sync,
{
    match io {
        IO::Http { req_template, dl_id, http_filter, group_by } => {
            let worker = &ctx.request_ctx.runtime.cmd_worker;
            let eval_http = EvalHttp::new(ctx, req_template, dl_id, group_by.clone());
            let request = eval_http.init_request()?;
            let value = match (&worker, http_filter) {
                (Some(worker), Some(http_filter)) => {
                    eval_http
                        .execute_with_worker(request, worker, http_filter)
                        .await?
                }
                _ => eval_http.execute(request).await?,
            };

            Ok(value)
        }
        IO::GraphQL { req_template, field_name, dl_id, .. } => {
            let value = ctx.value();

            if let Some(value) = value {
                if value.as_array().is_some() {
                    let loader =
                        GraphqlDataLoader::new(ctx.request_ctx.runtime.clone(), dl_id.is_some());
                    let mut requests_keys = Vec::new();
                    let mut requests = HashSet::new();

                    value.try_for_each(|value| {
                        let ctx = ctx.with_value(value);
                        let req = req_template.to_request(&ctx)?;

                        let dl_req = DataLoaderRequest::new(req, BTreeSet::default());
                        requests.insert(dl_req.clone());
                        requests_keys.push(dl_req);

                        anyhow::Ok(())
                    })?;

                    let requests: Vec<_> = requests.into_iter().collect();
                    let results = loader.load(&requests).await?;

                    let values = requests_keys
                        .iter()
                        .map(|key| {
                            let res = results.get(key).cloned().unwrap_or_default();

                            parse_graphql_response(ctx, res, field_name)
                        })
                        .collect::<Result<_, _>>()?;

                    return Ok(ConstValue::List(values));
                }
            }

            let req = req_template.to_request(ctx)?;

            let res = if ctx.request_ctx.upstream.batch.is_some()
                && matches!(req_template.operation_type, GraphQLOperationType::Query)
            {
                let data_loader: Option<&DataLoader<DataLoaderRequest, GraphqlDataLoader>> =
                    dl_id.and_then(|dl| ctx.request_ctx.gql_data_loaders.get(dl.as_usize()));
                execute_request_with_dl(ctx, req, data_loader).await?
            } else {
                execute_raw_request(ctx, req).await?
            };

            set_headers(ctx, &res);
            parse_graphql_response(ctx, res, field_name)
        }
        IO::Grpc { req_template, dl_id, group_by } => {
            let value = ctx.value();

            if let Some(value) = value {
                if value.as_array().is_some() {
                    let loader = GrpcDataLoader {
                        runtime: ctx.request_ctx.runtime.clone(),
                        operation: req_template.operation.clone(),
                        group_by: group_by.clone(),
                    };
                    let mut requests_keys = Vec::new();
                    let mut requests = HashSet::new();

                    value.try_for_each(|value| {
                        let ctx = ctx.with_value(value);
                        let req = req_template.render(&ctx)?;

                        let dl_req = grpc::DataLoaderRequest::new(req, BTreeSet::default());
                        requests.insert(dl_req.clone());
                        requests_keys.push(dl_req);

                        anyhow::Ok(())
                    })?;

                    let requests: Vec<_> = requests.into_iter().collect();
                    let results = loader.load(&requests).await?;

                    let values: Vec<_> = requests_keys
                        .iter()
                        .map(|key| {
                            let res = results.get(key).cloned().unwrap_or_default();

                            res.body
                        })
                        .collect();

                    return Ok(ConstValue::List(values));
                }
            }

            let rendered = req_template.render(ctx)?;

            let res = if ctx.request_ctx.upstream.batch.is_some() &&
                    // TODO: share check for operation_type for resolvers
                    matches!(req_template.operation_type, GraphQLOperationType::Query)
            {
                let data_loader: Option<&DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>> =
                    dl_id.and_then(|index| ctx.request_ctx.grpc_data_loaders.get(index.as_usize()));
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
