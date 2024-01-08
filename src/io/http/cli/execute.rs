use reqwest_middleware::ClientWithMiddleware;
use crate::http::Response;
use anyhow::Result;

pub async fn execute_raw(client: &ClientWithMiddleware, request: reqwest::Request) -> Result<Response<Vec<u8>>> {
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    let response = client.execute(request).await?;
    super::super::to_resp_raw(response).await
}

pub async fn execute(client: &ClientWithMiddleware, request: reqwest::Request) -> Result<Response<async_graphql::Value>> {
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    let response = client.execute(request).await?;
    super::super::to_resp_value(response).await
}