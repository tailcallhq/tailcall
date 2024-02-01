use std::pin::Pin;
use std::sync::Arc;

use futures_util::Future;
use hyper::body::Bytes;

use crate::channel::{JsRequest, JsResponse, Message, MessageContent};
use crate::http::Response;
use crate::{HttpIO, ScriptIO};

#[derive(Clone)]
pub struct HttpFilter {
    client: Arc<dyn HttpIO + Send + Sync>,
    script: Arc<dyn ScriptIO<Message, Message> + Send + Sync>,
}

impl HttpFilter {
    pub fn new(
        http: impl HttpIO + Send + Sync,
        script: impl ScriptIO<Message, Message> + Send + Sync + 'static,
    ) -> Self {
        HttpFilter { client: Arc::new(http), script: Arc::new(script) }
    }

    fn on_command<'a>(
        &'a self,
        command: Message,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Response<hyper::body::Bytes>>> + Send + 'a>>
    {
        Box::pin(async move {
            match command {
                Message { message: MessageContent::Request(request), id } => {
                    let request = request.try_into()?;
                    let response = self.client.execute(request).await?;
                    if id.is_none() {
                        return Ok(response);
                    }
                    let response = JsResponse::try_from(&response)?;
                    let command = self
                        .script
                        .on_event(Message { message: MessageContent::Response(response), id })
                        .await?;
                    Ok(self.on_command(command).await?)
                }
                Message { message: MessageContent::Response(response), id: _ } => {
                    let res: anyhow::Result<Response<Bytes>> = response.try_into();
                    res
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
        let request = JsRequest::try_from(&request)?;
        let command = self
            .script
            .on_event(Message { message: MessageContent::Request(request), id: None })
            .await?;
        self.on_command(command).await
    }
}
