use std::sync::Arc;

use tokio::runtime::Builder;
use tokio::spawn;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::oneshot;
use tokio::task::LocalSet;

use super::channel::Message;
use super::worker::{Worker, WorkerMessage};
use super::{JsRequest, JsResponse};
use crate::{blueprint, HttpIO, WorkerIO};

pub struct Runtime {
    sender: UnboundedSender<WorkerMessage>,
}

impl Runtime {
    pub fn new(script: blueprint::Script, http: Arc<dyn HttpIO>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<WorkerMessage>();
        let (http_sender, mut http_receiver) =
            mpsc::unbounded_channel::<(oneshot::Sender<JsResponse>, JsRequest)>();

        spawn(async move {
            while let Some((send_response, request)) = http_receiver.recv().await {
                let http = http.clone();

                spawn(async move {
                    let result = http.execute(request.try_into().unwrap()).await;
                    let response = result.and_then(JsResponse::try_from).unwrap();

                    send_response.send(response).unwrap();
                });
            }
        });

        // TODO: add support for multiple threads
        std::thread::spawn(move || {
            let rt = Builder::new_current_thread().enable_time().build().unwrap();
            let local = LocalSet::new();

            local.spawn_local(async move {
                let worker = Worker::new(script, http_sender).await?;
                worker.listen(receiver).await?;

                Ok::<_, anyhow::Error>(())
            });

            rt.block_on(local);
        });

        Self { sender }
    }
}

#[async_trait::async_trait]
impl WorkerIO<Message, Message> for Runtime {
    async fn dispatch(&self, event: Message) -> anyhow::Result<Message> {
        log::debug!("event: {:?}", event);
        let (tx, rx) = oneshot::channel();

        self.sender.send((tx, event))?;

        Ok(rx.await?)
    }
}
