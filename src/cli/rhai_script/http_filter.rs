use std::pin::Pin;
use std::sync::Arc;

use async_graphql_value::ConstValue;
use futures_util::Future;
use url::Url;

use crate::http::Response;
use crate::{HttpIO, WorkerIO};

#[derive(Debug, Clone)]
pub struct Request<Body: Default + Clone> {
    pub method: reqwest::Method,
    pub url: Url,
    pub headers: reqwest::header::HeaderMap,
    pub body: Option<Body>,
}

impl From<reqwest::Request> for Request<ConstValue> {
    fn from(req: reqwest::Request) -> Self {
        Request {
            method: req.method().clone(),
            url: req.url().clone(),
            headers: req.headers().clone(),
            body: req.body().and_then(|body| {
                let json = body
                    .as_bytes()
                    .and_then(|bytes| serde_json::from_slice::<ConstValue>(bytes).ok());
                json
            }),
        }
    }
}

impl Request<ConstValue> {
    pub fn try_into(self) -> anyhow::Result<reqwest::Request> {
        let mut req = reqwest::Request::new(self.method, self.url);
        *req.headers_mut() = self.headers;
        if let Some(body) = self.body {
            let bytes = body.into_json()?.to_string().into_bytes();
            *req.body_mut() = Some(reqwest::Body::from(bytes));
        }
        Ok(req)
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub message: MessageContent,
    pub id: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum MessageContent {
    Request(Request<ConstValue>),
    Response(Response<ConstValue>),
    Empty,
}

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
                    let command = self.worker.dispatch(Message {
                        message: MessageContent::Response(response.to_json()?),
                        id,
                    })?;
                    Ok(self.on_command(command).await?)
                }
                Message { message: MessageContent::Response(response), id: _ } => {
                    Ok(response.to_bytes()?)
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
            .dispatch(Message { message: MessageContent::Request(request.into()), id: None })?;
        self.on_command(command).await
    }
}
