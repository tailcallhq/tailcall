use tokio::runtime::Builder;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::oneshot;
use tokio::task::LocalSet;

use super::channel::Message;
use super::worker::{Worker, WorkerMessage};
use crate::{blueprint, WorkerIO};

pub struct Runtime {
    sender: UnboundedSender<WorkerMessage>,
}

impl Runtime {
    pub fn new(script: blueprint::Script) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<WorkerMessage>();

        // TODO: add support for multiple threads
        std::thread::spawn(move || {
            let rt = Builder::new_current_thread().enable_time().build().unwrap();
            let local = LocalSet::new();

            local.spawn_local(async move {
                let worker = Worker::new(script).await.unwrap();
                worker.listen(receiver).await.unwrap();
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
