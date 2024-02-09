use std::sync::Arc;

use tokio::runtime::Builder;
use tokio::spawn;
use tokio::task::LocalSet;

use super::channel::{wait_channel, Message, WaitSender};
use super::worker::Worker;
use super::{JsRequest, JsResponse};
use crate::{blueprint, HttpIO, WorkerIO};

pub struct Runtime {
    work_sender: WaitSender<Message, Message>,
}

impl Runtime {
    pub fn new(script: blueprint::Script, http: Arc<dyn HttpIO>) -> Self {
        let (work_sender, work_receiver) = wait_channel::<Message, Message>();
        let (http_sender, mut http_receiver) = wait_channel::<JsRequest, JsResponse>();

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
                worker.listen(work_receiver).await?;

                Ok::<_, anyhow::Error>(())
            });

            rt.block_on(local);
        });

        Self { work_sender }
    }
}

#[async_trait::async_trait]
impl WorkerIO<Message, Message> for Runtime {
    async fn dispatch(&self, event: Message) -> anyhow::Result<Message> {
        log::debug!("event: {:?}", event);
        self.work_sender.wait_send(event).await
    }
}
