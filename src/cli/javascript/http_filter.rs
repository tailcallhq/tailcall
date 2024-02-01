use std::pin::Pin;
use std::sync::Arc;

use futures_util::Future;
use hyper::body::Bytes;

use crate::channel::{Command, Event, JsRequest, JsResponse, Message};
use crate::http::Response;
use crate::{HttpIO, ScriptIO};

#[derive(Clone)]
pub struct HttpFilter {
    client: Arc<dyn HttpIO + Send + Sync>,
    script: Arc<dyn ScriptIO<Event, Command> + Send + Sync>,
}

impl HttpFilter {
    pub fn new(
        http: impl HttpIO + Send + Sync,
        script: impl ScriptIO<Event, Command> + Send + Sync + 'static,
    ) -> Self {
        HttpFilter { client: Arc::new(http), script: Arc::new(script) }
    }

    fn on_command<'a>(
        &'a self,
        command: Command,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Response<hyper::body::Bytes>>> + Send + 'a>>
    {
        Box::pin(async move {
            match command {
                Command { message: Message::Request(request), id } => {
                    let request = request.try_into()?;
                    let response = self.client.execute(request).await?;
                    let response = JsResponse::try_from(&response)?;
                    let command = self
                        .script
                        .on_event(Event { message: Message::Response(response), id })
                        .await?;
                    Ok(self.on_command(command).await?)
                }
                Command { message: Message::Response(response), id: _ } => {
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
            .on_event(Event { message: Message::Request(request), id: None })
            .await?;
        self.on_command(command).await
    }
}
