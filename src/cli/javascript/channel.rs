use std::{any, future::IntoFuture, sync::Arc, vec};

use futures_util::future::join_all;
use serde::{Deserialize, Serialize};

use crate::{cli::command, http::Response, HttpIO, WorkerIO};

use super::{JsRequest, JsResponse};

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
        worker: impl WorkerIO<Event, Command> + Send + Sync + 'static,
        client: impl HttpIO + Send + Sync + 'static,
    ) -> Self {
        Self { worker: Arc::new(worker), client: Arc::new(client) }
    }

    async fn dispatch(&mut self, request: JsRequest) -> anyhow::Result<JsResponse> {
        let event = Event::Request(request);
        self.on_event(event).await
    }

    async fn on_event(&self, event: Event) -> anyhow::Result<JsResponse> {
        let commands = self.worker.dispatch(event)?;
        join_all(commands.into_iter().map(|command| self.on_command(command)))
            .await
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("No response"))?
    }

    #[async_recursion::async_recursion]
    async fn on_command(&self, command: Command) -> anyhow::Result<JsResponse> {
        match command {
            Command::Request(Continue { message, id }) => {
                let response = self.client.execute(message.try_into()?).await?;
                let event = Event::Response(Continue { message: response.try_into()?, id });
                self.on_event(event).await
            }
            Command::Response(response) => Ok(response),
        }
    }
}

#[async_trait::async_trait]
impl HttpIO for Channel {
    async fn execute(
        &self,
        request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>> {
        self.dispatch(request.try_into()?).await?.try_into()
    }
}
