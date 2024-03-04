
use std::sync::Arc;

use async_graphql_value::ConstValue;

use url::Url;

use crate::http::Response;
use crate::{HttpIO, WorkerIO};

#[derive(Debug, Clone)]
pub struct HttpRequest<Body: Default + Clone> {
    pub method: reqwest::Method,
    pub url: Url,
    pub headers: reqwest::header::HeaderMap,
    pub body: Option<Body>,
}

impl From<&reqwest::Request> for HttpRequest<ConstValue> {
    fn from(req: &reqwest::Request) -> Self {
        HttpRequest {
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

impl HttpRequest<ConstValue> {
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
pub enum Event {
    Request(HttpRequest<ConstValue>),
}

impl Event {
    pub fn get_request(self) -> HttpRequest<ConstValue> {
        match self {
            Event::Request(request) => request,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Request(HttpRequest<ConstValue>),
    Response(Response<ConstValue>),
}

impl Command {
    pub fn new_request(request: HttpRequest<ConstValue>) -> Self {
        Command::Request(request)
    }
    pub fn new_response(response: Response<ConstValue>) -> Self {
        Command::Response(response)
    }
}

#[derive(Clone)]
pub struct RequestFilter {
    worker: Arc<dyn WorkerIO<Event, Command>>,
    client: Arc<dyn HttpIO>,
}

impl RequestFilter {
    pub fn new(
        client: impl HttpIO + Send + Sync + 'static,
        worker: impl WorkerIO<Event, Command>,
    ) -> Self {
        RequestFilter { client: Arc::new(client), worker: Arc::new(worker) }
    }

    #[async_recursion::async_recursion]
    async fn on_request(
        &self,
        mut request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>> {
        let js_request = HttpRequest::from(&request);
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
                            .set_path(js_response.headers.get("location").unwrap().to_str()?);
                        self.on_request(request).await
                    } else {
                        Ok(js_response.to_bytes()?)
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
