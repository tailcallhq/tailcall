use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

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

pub type CallbackMessage<I, O> = (oneshot::Sender<O>, I);

pub trait CallbackSender<I, O> {
    async fn send_with_callback(&self, input: I) -> Result<O>;
}

impl<I: Send + Sync + 'static, O: Send + Sync + 'static> CallbackSender<I, O>
    for mpsc::UnboundedSender<CallbackMessage<I, O>>
{
    async fn send_with_callback(&self, input: I) -> Result<O> {
        let (tx, rx) = oneshot::channel::<O>();

        self.send((tx, input))?;

        Ok(rx.await?)
    }
}

impl<I: Send + Sync + 'static, O: Send + Sync + 'static> CallbackSender<I, O>
    for loole::Sender<CallbackMessage<I, O>>
{
    async fn send_with_callback(&self, input: I) -> Result<O> {
        let (tx, rx) = oneshot::channel::<O>();

        self.send_async((tx, input)).await?;

        Ok(rx.await?)
    }
}
