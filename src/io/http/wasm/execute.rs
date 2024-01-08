use anyhow::Result;
use reqwest::Request;
use reqwest_middleware::ClientWithMiddleware;

use crate::http::Response;
pub async fn execute_raw(client: &ClientWithMiddleware, request: Request) -> Result<Response<Vec<u8>>> {
  async_std::task::spawn_local(internal_execute_raw(client.clone(), request)).await
}

pub async fn execute(client: &ClientWithMiddleware, request: Request) -> Result<Response<async_graphql::Value>> {
  async_std::task::spawn_local(internal_execute_val(client.clone(), request)).await
}

async fn internal_execute_raw(client: ClientWithMiddleware, request: Request) -> Result<Response<Vec<u8>>> {
  let response = client.execute(request).await?;
  super::super::to_resp_raw(response).await
}

async fn internal_execute_val(
  client: ClientWithMiddleware,
  request: Request,
) -> Result<Response<async_graphql::Value>> {
  let response = client.execute(request).await?;
  super::super::to_resp_value(response).await
}
