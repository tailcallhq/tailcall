use std::{
    collections::{BTreeSet, HashSet},
    sync::Arc,
};

use async_graphql::from_value;
use async_graphql_value::ConstValue;
use reqwest::Request;

use super::model::DataLoaderId;
use super::{EvalContext, ResolverContextLike};
use crate::core::grpc::protobuf::ProtobufOperation;
use crate::core::grpc::request::execute_grpc_request;
use crate::core::grpc::request_template::RenderedRequestTemplate;
use crate::core::http::{
    cache_policy, DataLoaderRequest, HttpDataLoader, HttpFilter, RequestTemplate, Response,
};
use crate::core::ir::Error;
use crate::core::json::JsonLike;
use crate::core::valid::Validator;
use crate::core::{
    config::group_by::GroupBy,
    data_loader::{DataLoader, Loader},
};
use crate::core::{grpc, http, worker, WorkerIO};

///
/// Executing a HTTP request is a bit more complex than just sending a request
/// and getting a response. There are optimizations and customizations that the
/// user might have configured. HttpRequestExecutor is responsible for handling
/// all of that.
pub struct EvalHttp<'a, 'ctx, Context: ResolverContextLike + Sync> {
    evaluation_ctx: &'ctx EvalContext<'a, Context>,
    data_loader: Option<&'a DataLoader<DataLoaderRequest, HttpDataLoader>>,
    request_template: &'a http::RequestTemplate,
    group_by: Option<GroupBy>,
}

impl<'a, 'ctx, Context: ResolverContextLike + Sync> EvalHttp<'a, 'ctx, Context> {
    pub fn new(
        evaluation_ctx: &'ctx EvalContext<'a, Context>,
        request_template: &'a RequestTemplate,
        id: &Option<DataLoaderId>,
        group_by: Option<GroupBy>,
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

        Self { evaluation_ctx, data_loader, request_template, group_by }
    }

    pub fn init_request(&self) -> Result<Request, Error> {
        Ok(self.request_template.to_request(self.evaluation_ctx)?)
    }

    pub async fn execute(&self, req: Request) -> Result<ConstValue, Error> {
        let ctx = &self.evaluation_ctx;

        let value = ctx.value();

        if let Some(value) = value {
            if value.as_array().is_some() {
                let req_template = &self.request_template;
                let loader = HttpDataLoader::new(ctx.request_ctx.runtime.clone(), self.group_by.clone(), false);
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

        Ok(response.body)
    }

    #[async_recursion::async_recursion]
    pub async fn execute_with_worker(
        &self,
        mut request: reqwest::Request,
        worker: &Arc<dyn WorkerIO<worker::Event, worker::Command>>,
        http_filter: &HttpFilter,
    ) -> Result<ConstValue, Error> {
        let js_request = worker::WorkerRequest::try_from(&request)?;
        let event = worker::Event::Request(js_request);

        let command = worker.call(&http_filter.on_request, event).await?;

        match command {
            Some(command) => match command {
                worker::Command::Request(w_request) => {
                    let response = self.execute(w_request.into()).await?;
                    Ok(response)
                }
                worker::Command::Response(w_response) => {
                    // Check if the response is a redirect
                    if (w_response.status() == 301 || w_response.status() == 302)
                        && w_response.headers().contains_key("location")
                    {
                        request
                            .url_mut()
                            .set_path(w_response.headers()["location"].as_str());
                        self.execute_with_worker(request, worker, http_filter).await
                    } else {
                        let response: Response<_> = w_response.try_into()?;
                        Ok(response.body)
                    }
                }
            },
            None => self.execute(request).await,
        }
    }
}

pub async fn execute_request_with_dl<
    'ctx,
    Ctx: ResolverContextLike,
    Dl: Loader<DataLoaderRequest, Value = Response<async_graphql::Value>, Error = Arc<anyhow::Error>>,
>(
    ctx: &EvalContext<'ctx, Ctx>,
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
