use std::sync::Arc;

use crate::core::http::Response;

use super::{
    ir::Error,
    worker::{self, Command, Event, WorkerRequest, WorkerResponse},
    WorkerIO,
};

#[derive(Clone)]
/// User can configure the hooks on directive
/// for the requests.
pub struct JsHooks {
    pub on_request: Option<String>,
    pub on_response: Option<String>,
}

impl JsHooks {
    pub fn new(
        on_request: Option<String>,
        on_response: Option<String>,
    ) -> Result<Self, &'static str> {
        if on_request.is_none() && on_response.is_none() {
            Err("At least one of on_request or on_response must be present")
        } else {
            Ok(Self { on_request, on_response })
        }
    }

    pub async fn on_request(
        &self,
        worker: &Arc<dyn WorkerIO<worker::Event, worker::Command>>,
        request: &reqwest::Request,
    ) -> Result<Option<worker::Command>, Error> {
        match &self.on_request {
            Some(on_request) => {
                let js_request = WorkerRequest::try_from(request)?;
                let event = worker::Event::Request(js_request);
                worker.call(on_request, event).await.map_err(|e| e.into())
            }
            None => Ok(None),
        }
    }

    pub async fn on_response(
        &self,
        worker: &Arc<dyn WorkerIO<worker::Event, worker::Command>>,
        response: Response<async_graphql::Value>,
    ) -> Result<Response<async_graphql::Value>, Error> {
        if let Some(on_response) = self.on_response.as_ref() {
            let js_response = WorkerResponse::try_from(response.clone())?;
            let response_event = Event::Response(js_response);
            let command = worker.call(on_response, response_event).await?;
            Ok(match command {
                Some(Command::Response(w_response)) => w_response.try_into()?,
                _ => response,
            })
        } else {
            Ok(response)
        }
    }
}
