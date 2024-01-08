use crate::http::Response;
use reqwest::IntoUrl;
use anyhow::Result;
pub async fn get_raw<T: IntoUrl>(url: T) -> Result<Response<Vec<u8>>> {
    let response = reqwest::get(url).await?;
    super::super::to_resp_raw(response).await
}

pub async fn get_string<T: IntoUrl>(url: T) -> Result<Response<String>> {
    let response = reqwest::get(url).await?;
    super::super::to_resp_string(response).await
}

pub async fn get_value<T: IntoUrl>(url: T) -> Result<Response<async_graphql::Value>> {
    let response = reqwest::get(url).await?;
    super::super::to_resp_value(response).await
}