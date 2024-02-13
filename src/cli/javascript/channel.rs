use std::convert::identity;
use std::sync::Arc;

use futures_util::future::join_all;
use serde::{Deserialize, Serialize};

use super::{JsRequest, JsResponse};
use crate::http::Response;
use crate::{HttpIO, WorkerIO};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub message: MessageContent,
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum MessageContent {
    Request(JsRequest),
    Response(JsResponse),
    Empty,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Continue<A> {
    message: A,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Request(JsRequest),
    Response(Continue<JsResponse>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    Request(Continue<JsRequest>),
    Response(JsResponse),
}

pub struct Channel {
    worker: Arc<dyn WorkerIO<Event, Command>>,
    client: Arc<dyn HttpIO>,
}

impl Channel {
    pub fn new(
        client: impl HttpIO + Send + Sync + 'static,
        worker: impl WorkerIO<Event, Command>,
    ) -> Self {
        Self { worker: Arc::new(worker), client: Arc::new(client) }
    }

    async fn dispatch(&self, request: JsRequest) -> anyhow::Result<Option<JsResponse>> {
        let event = Event::Request(request);
        let response = self.on_event(event).await?;
        Ok(response)
    }

    async fn on_event(&self, event: Event) -> anyhow::Result<Option<JsResponse>> {
        log::debug!("event: {:?}", event);
        let worker = self.worker.clone();
        let commands = worker.dispatch(event).await?;
        let responses = join_all(commands.into_iter().map(|command| self.on_command(command)))
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(responses.into_iter().find_map(identity))
    }

    #[async_recursion::async_recursion]
    async fn on_command(&self, command: Command) -> anyhow::Result<Option<JsResponse>> {
        log::debug!("command: {:?}", command);
        match command {
            Command::Request(Continue { message, id }) => {
                let response = self.client.execute(message.try_into()?).await?;
                let event = Event::Response(Continue { message: response.try_into()?, id });
                self.on_event(event).await
            }
            Command::Response(response) => Ok(Some(response)),
        }
    }
}

#[async_trait::async_trait]
impl HttpIO for Channel {
    async fn execute(
        &self,
        request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>> {
        let js_request = JsRequest::try_from(&request)?;
        let response = self
            .dispatch(js_request)
            .await?
            .ok_or(anyhow::anyhow!("No response"))?;
        response.try_into()
    }
}
