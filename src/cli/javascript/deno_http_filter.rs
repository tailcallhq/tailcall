use std::pin::Pin;
use std::sync::Arc;

use futures_util::Future;

use super::deno_channel::{JsResponse, Message, MessageContent};
use crate::http::Response;
use crate::{HttpIO, WorkerIO};

#[derive(Clone)]
pub struct HttpFilter {
    client: Arc<dyn HttpIO + Send + Sync>,
    worker: Arc<dyn WorkerIO<Message, Message> + Send + Sync>,
}

impl HttpFilter {
    pub fn new(
        http: impl HttpIO + Send + Sync,
        script: impl WorkerIO<Message, Message> + Send + Sync + 'static,
    ) -> Self {
        HttpFilter { client: Arc::new(http), worker: Arc::new(script) }
    }

    fn on_command<'a>(
        &'a self,
        command: Message,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Response<hyper::body::Bytes>>> + Send + 'a>>
    {
        Box::pin(async move {
            match command {
                Message { message: MessageContent::Request(request), id } => {
                    let request = request;
                    let response = self.client.execute(request.try_into()?).await?;
                    if id.is_none() {
                        return Ok(response);
                    }
                    let command = self
                        .worker
                        .dispatch(Message {
                            message: MessageContent::Response(JsResponse::try_from(response)?),
                            id,
                        })
                        .await?;
                    Ok(self.on_command(command).await?)
                }
                Message { message: MessageContent::Response(response), id: _ } => {
                    Ok(response.try_into()?)
                }
                Message { message: MessageContent::Empty, id: _ } => {
                    anyhow::bail!("No response received from worker")
                }
            }
        })
    }
}

#[async_trait::async_trait]
impl HttpIO for HttpFilter {
    async fn execute(
        &self,
        request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>> {
        let command = self
            .worker
            .dispatch(Message {
                message: MessageContent::Request(request.try_into()?),
                id: None,
            })
            .await?;
        self.on_command(command).await
    }
}
