#[cfg(feature = "default")]
pub mod cli;
#[cfg(not(feature = "default"))]
pub mod wasm;

#[cfg(not(feature = "default"))]
pub use wasm::*;
#[cfg(feature = "default")]
pub use cli::*;

use crate::http::Response;


pub(super) async fn to_resp_raw(response: reqwest::Response) -> anyhow::Result<Response<Vec<u8>>> {
    let resp = Response::from_response_to_vec(response).await?;
    Ok(resp)
}

pub(super) async fn to_resp_value(response: reqwest::Response) -> anyhow::Result<Response<async_graphql::Value>> {
    let resp = Response::from_response_to_val(response).await?;
    Ok(resp)
}

pub(super) async fn to_resp_string(response: reqwest::Response) ->anyhow:: Result<Response<String>> {
    let resp = Response::from_response_to_string(response).await?;
    Ok(resp)
}