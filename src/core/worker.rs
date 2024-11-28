use std::sync::Arc;

use super::ir::Error;
use super::worker::WorkerRequest;
use super::{worker, WorkerIO};
use crate::core::http::Response;

/// User can configure the hooks on directive
/// for the requests.
#[derive(Clone, Debug)]
pub struct WorkerHooks {
    pub on_request: Option<String>,
    pub on_response: Option<String>,
}

impl WorkerHooks {
    pub fn try_new(
        on_request: Option<String>,
        on_response: Option<String>,
    ) -> Result<Self, &'static str> {
        if on_request.is_none() && on_response.is_none() {
            Err("At least one of on_request or on_response must be present")
        } else {
            Ok(Self { on_request, on_response })
        }
    }

    /// on request hook called before the request is sent and it sends the
    /// request to the worker for modification.
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

    /// on response hook called after the response is received and it sends the
    /// response body to the worker and returns the response.
    pub async fn on_response(
        &self,
        worker: &Arc<
            dyn WorkerIO<async_graphql_value::ConstValue, async_graphql_value::ConstValue>,
        >,
        response: Response<async_graphql::Value>,
    ) -> Result<Response<async_graphql::Value>, Error> {
        if let Some(on_response) = self.on_response.as_ref() {
            match worker.call(on_response, response.body.clone()).await? {
                Some(js_response) => Ok(response.body(js_response)),
                None => Ok(response),
            }
        } else {
            Ok(response)
        }
    }
}
