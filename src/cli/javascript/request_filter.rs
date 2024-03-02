use std::sync::Arc;

use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

use super::{JsRequest, JsResponse};
use crate::http::Response;
use crate::{HttpIO, WorkerIO};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Request(JsRequest),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    Request(JsRequest),
    Response(JsResponse),
}

pub struct RequestFilter {
    worker: Arc<dyn WorkerIO<Event, Command>>,
    client: Arc<dyn HttpIO>,
}

impl RequestFilter {
    pub fn new(
        client: impl HttpIO + Send + Sync + 'static,
        worker: impl WorkerIO<Event, Command>,
    ) -> Self {
        Self { worker: Arc::new(worker), client: Arc::new(client) }
    }

    #[async_recursion::async_recursion]
    async fn on_request(&self, mut request: reqwest::Request) -> anyhow::Result<Response<Bytes>> {
        let js_request = JsRequest::try_from(&request)?;
        let event = Event::Request(js_request);
        let command = self.worker.call("onRequest".to_string(), event).await?;
        match command {
            Some(command) => match command {
                Command::Request(js_request) => {
                    let response = self.client.execute(js_request.try_into()?).await?;
                    Ok(response)
                }
                Command::Response(js_response) => {
                    // Check if the response is a redirect
                    if (js_response.status == 301 || js_response.status == 302)
                        && js_response.headers.contains_key("location")
                    {
                        request
                            .url_mut()
                            .set_path(js_response.headers["location"].as_str());
                        self.on_request(request).await
                    } else {
                        Ok(js_response.try_into()?)
                    }
                }
            },
            None => self.client.execute(request).await,
        }
    }
}

#[async_trait::async_trait]
impl HttpIO for RequestFilter {
    async fn execute(
        &self,
        request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>> {
        self.on_request(request).await
    }
}
