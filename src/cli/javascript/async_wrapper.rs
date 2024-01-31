use tokio::{
    runtime::Builder,
    spawn,
    sync::{mpsc, oneshot},
    task::{spawn_local, LocalSet},
};

use crate::{blueprint, http::Response, HttpIO};

use super::worker::Worker;

pub type ChannelResult = anyhow::Result<Response<hyper::body::Bytes>>;
pub type ChannelMessage = (oneshot::Sender<ChannelResult>, reqwest::Request);

#[derive(Debug, Clone)]
pub struct JsTokioWrapper {
    sender: mpsc::UnboundedSender<ChannelMessage>,
}

impl JsTokioWrapper {
    pub fn new(script: blueprint::Script, http: impl HttpIO) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<ChannelMessage>();
        let (http_sender, mut http_receiver) = mpsc::unbounded_channel::<ChannelMessage>();

        spawn(async move {
            while let Some((send_response, request)) = http_receiver.recv().await {
                let result = http.execute(request).await;

                send_response.send(result).unwrap();
            }
        });

        std::thread::spawn(move || {
            let rt = Builder::new_current_thread().enable_all().build().unwrap();
            let local = LocalSet::new();

            local.spawn_local(async move {
                let worker = Worker::new(script, http_sender).unwrap();

                while let Some((response, request)) = receiver.recv().await {
                    let worker = worker.clone();
                    spawn_local(async move {
                        let result = worker.on_event(request).await;

                        // ignore errors
                        let _ = response.send(result);
                    });
                }
            });

            rt.block_on(local);
        });

        Self { sender }
    }
}

#[async_trait::async_trait]
impl HttpIO for JsTokioWrapper {
    async fn execute(
        &self,
        request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>> {
        let (tx, rx) = oneshot::channel();

        self.sender.send((tx, request))?;

        rx.await?
    }
}
