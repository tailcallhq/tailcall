use std::sync::Arc;

use async_graphql::ServerError;
use hyper::{Body, Request, Response, StatusCode};
use serde::de::DeserializeOwned;

use crate::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};
use crate::http::parser::Parser;
use crate::http::request_handlers::request_handler::{
  create_request_context, update_cache_control_header, update_response_headers,
};
use crate::http::ServerContext;

pub async fn api_request<T: DeserializeOwned + GraphQLRequestLike>(
  req: Request<Body>,
  server_ctx: &ServerContext,
) -> anyhow::Result<Response<Body>> {
  if let Some(query) = req.uri().path_and_query() {
    let mut parser = Parser::from_uri(req.uri())?;
    let request = parser.parse::<T>(&server_ctx.blueprint.definitions);
    match request {
      Ok(request) => {
        let req_ctx = Arc::new(create_request_context(&req, server_ctx));
        let response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;
        let mut response = update_cache_control_header(response, server_ctx, req_ctx).to_response()?;
        update_response_headers(&mut response, server_ctx);
        Ok(response)
      }
      Err(err) => {
        log::error!("Failed to parse request: {query}");
        let status_code = if err.to_string().starts_with("404") {
          StatusCode::NOT_FOUND
        } else {
          StatusCode::BAD_REQUEST
        };
        let err = ServerError::new(format!("Unexpected API Request: {}", err), None);
        let mut resp = async_graphql::Response::default();
        resp.errors = vec![err];
        Ok(GraphQLResponse::from(resp).to_response_with_status_code(status_code)?)
      }
    }
  } else {
    log::error!("Failed to parse request, invalid url",);
    let err = ServerError::new("Failed to parse request, invalid url".to_string(), None);
    let mut resp = async_graphql::Response::default();
    resp.errors = vec![err];
    Ok(GraphQLResponse::from(resp).to_response_with_status_code(StatusCode::BAD_REQUEST)?)
  }
}
