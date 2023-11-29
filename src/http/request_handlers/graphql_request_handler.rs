use std::sync::Arc;

use anyhow::Result;
use async_graphql::ServerError;
use hyper::{Body, Request, Response};
use serde::de::DeserializeOwned;

use crate::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};
use crate::http::request_handlers::request_handler::{
  create_request_context, update_cache_control_header, update_response_headers,
};
use crate::http::ServerContext;

pub async fn graphql_request<T: DeserializeOwned + GraphQLRequestLike>(
  req: Request<Body>,
  server_ctx: &ServerContext,
) -> Result<Response<Body>> {
  let req_ctx = Arc::new(create_request_context(&req, server_ctx));
  let bytes = hyper::body::to_bytes(req.into_body()).await?;
  let request = serde_json::from_slice::<T>(&bytes);
  match request {
    Ok(request) => {
      let mut response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;
      response = update_cache_control_header(response, server_ctx, req_ctx);
      let mut resp = response.to_response()?;
      update_response_headers(&mut resp, server_ctx);
      Ok(resp)
    }
    Err(err) => {
      log::error!(
        "Failed to parse request: {}",
        String::from_utf8(bytes.to_vec()).unwrap()
      );

      let mut response = async_graphql::Response::default();
      let server_error = ServerError::new(format!("Unexpected GraphQL Request: {}", err), None);
      response.errors = vec![server_error];

      Ok(GraphQLResponse::from(response).to_response()?)
    }
  }
}
