use anyhow::Result;
use reqwest::{Client, IntoUrl, Request};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use crate::config::Upstream;
use crate::http::{HttpClientOptions, Response};

pub fn make_client(_: &Upstream, _: HttpClientOptions) -> ClientWithMiddleware {
  let builder = Client::builder();
  let client = ClientBuilder::new(builder.build().expect("Failed to build client"));
  client.build()
}

pub async fn execute_raw(client: &ClientWithMiddleware, request: Request) -> Result<Response<Vec<u8>>> {
  async_std::task::spawn_local(internal_execute_raw(client.clone(), request)).await
}

pub async fn execute(client: &ClientWithMiddleware, request: Request) -> Result<Response<async_graphql::Value>> {
  async_std::task::spawn_local(internal_execute_val(client.clone(), request)).await
}

async fn internal_execute_raw(client: ClientWithMiddleware, request: Request) -> Result<Response<Vec<u8>>> {
  let response = client.execute(request).await?;
  super::to_resp_raw(response).await
}

async fn internal_execute_val(
  client: ClientWithMiddleware,
  request: Request,
) -> Result<Response<async_graphql::Value>> {
  let response = client.execute(request).await?;
  super::to_resp_value(response).await
}

pub async fn get_raw<T: IntoUrl + 'static>(url: T) -> Result<Response<Vec<u8>>> {
  async_std::task::spawn_local(internal_get_raw(url)).await
}

pub async fn get_string<T: IntoUrl + 'static>(url: T) -> Result<Response<String>> {
  async_std::task::spawn_local(internal_get_string(url)).await
}

pub async fn get_value<T: IntoUrl + 'static>(url: T) -> Result<Response<async_graphql::Value>> {
  async_std::task::spawn_local(internal_get_val(url)).await
}

async fn internal_get_raw<T: IntoUrl + 'static>(url: T) -> Result<Response<Vec<u8>>> {
  let response = reqwest::get(url).await?;
  super::to_resp_raw(response).await
}

async fn internal_get_val<T: IntoUrl + 'static>(url: T) -> Result<Response<async_graphql::Value>> {
  let response = reqwest::get(url).await?;
  super::to_resp_value(response).await
}

async fn internal_get_string<T: IntoUrl + 'static>(url: T) -> Result<Response<String>> {
  let response = reqwest::get(url).await?;
  super::to_resp_string(response).await
}
