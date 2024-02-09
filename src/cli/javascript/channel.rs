use std::ops::{Deref, DerefMut};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    oneshot,
};

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

type WaitMessage<I, O> = (oneshot::Sender<O>, I);

pub struct WaitSender<I, O>(UnboundedSender<WaitMessage<I, O>>);

pub struct WaitReceiver<I, O>(UnboundedReceiver<WaitMessage<I, O>>);

impl<I, O> Deref for WaitSender<I, O> {
    type Target = UnboundedSender<WaitMessage<I, O>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I, O> Clone for WaitSender<I, O> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I, O> Deref for WaitReceiver<I, O> {
    type Target = UnboundedReceiver<WaitMessage<I, O>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I, O> DerefMut for WaitReceiver<I, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I: Send + Sync + 'static, O: Send + Sync + 'static> WaitSender<I, O> {
    pub async fn wait_send(&self, input: I) -> Result<O> {
        let (tx, rx) = oneshot::channel::<O>();

        self.0.send((tx, input))?;

        Ok(rx.await?)
    }
}

pub fn wait_channel<I, O>() -> (WaitSender<I, O>, WaitReceiver<I, O>) {
    let (tx, rx) = mpsc::unbounded_channel();

    (WaitSender(tx), WaitReceiver(rx))
}
