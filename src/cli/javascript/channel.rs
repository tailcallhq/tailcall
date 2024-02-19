use serde::{Deserialize, Serialize};

use super::{JsRequest, JsResponse};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub message: MessageContent,
    pub id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum MessageContent {
    Request(JsRequest),
    Response(JsResponse),
    Empty,
}
