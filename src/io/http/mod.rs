#[cfg(feature = "default")]
pub mod native;
#[cfg(not(feature = "default"))]
pub mod wasm;

#[cfg(feature = "default")]
pub use native::*;
#[cfg(not(feature = "default"))]
pub use wasm::*;

use crate::http::Response;

// TODO: there is no method to change the version in reqwest::wasm

#[cfg(feature = "default")]
pub fn set_req_version(req: &mut reqwest::Request) {
  *req.version_mut() = reqwest::Version::HTTP_2;
}

#[cfg(not(feature = "default"))]
pub fn set_req_version(_: &mut reqwest::Request) {}

pub(super) async fn to_resp_raw(response: reqwest::Response) -> anyhow::Result<Response<Vec<u8>>> {
  let resp = Response::from_response_to_vec(response).await?;
  Ok(resp)
}

pub(super) async fn to_resp_value(response: reqwest::Response) -> anyhow::Result<Response<async_graphql::Value>> {
  let resp = Response::from_response_to_val(response).await?;
  Ok(resp)
}

pub(super) async fn to_resp_string(response: reqwest::Response) -> anyhow::Result<Response<String>> {
  let resp = Response::from_response_to_string(response).await?;
  Ok(resp)
}
