use reqwest::IntoUrl;
use crate::http::Response;
use anyhow::Result;
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
    super::super::to_resp_raw(response).await
}

async fn internal_get_val<T: IntoUrl + 'static>(url: T) -> Result<Response<async_graphql::Value>> {
    let response = reqwest::get(url).await?;
    super::super::to_resp_value(response).await
}

async fn internal_get_string<T: IntoUrl + 'static>(url: T) -> Result<Response<String>> {
    let response = reqwest::get(url).await?;
    super::super::to_resp_string(response).await
}