use std::sync::Arc;

use hyper::body::Bytes;
use mini_v8::{MiniV8, Value, Values};

use super::serde_v8::SerdeV8;
use crate::channel::{JsRequest, JsResponse};
use crate::http::Response;
use crate::{blueprint, HttpIO, ToAnyHow};

struct Worker {
    v8: MiniV8,
    http: Arc<dyn HttpIO>,
    closure: mini_v8::Function,
}

fn create_closure(script: &str) -> String {
    format!("(function() {{{} return onEvent}})();", script)
}

impl Worker {
    fn new(
        script: blueprint::Script,
        v8: &mini_v8::MiniV8,
        http: impl HttpIO,
    ) -> anyhow::Result<Self> {
        let _ = super::shim::init(v8);
        let script = mini_v8::Script {
            source: create_closure(script.source.as_str()),
            timeout: script.timeout,
            ..Default::default()
        };
        let value: mini_v8::Value = v8
            .eval(script)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let closure = value
            .as_function()
            .ok_or_else(|| anyhow::anyhow!("expected an 'onEvent' function"))?
            .clone();

        Ok(Self { v8: v8.clone(), http: Arc::new(http), closure })
    }

    pub async fn on_event(&self, request: reqwest::Request) -> anyhow::Result<Response<Bytes>> {
        let js_request = JsRequest::try_from(&request)?;
        let js_request_v8 = js_request.to_v8(&self.v8)?;
        // Initiate an async call
        let result = self
            .closure
            .call::<Values, Value>(Values::from_iter(vec![js_request_v8]))
            .or_anyhow("failed to dispatch request to js-worker: ")?;

        // Check if the result is a response
        let js_response = JsResponse::from_v8(&result)?;

        let response = Response::<Bytes>::try_from(js_response)?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::*;
    use crate::blueprint::Script;
    use crate::cli::NativeHttp;
    use pretty_assertions::assert_eq;

    fn new_worker(script: &str) -> anyhow::Result<Worker> {
        let v8 = mini_v8::MiniV8::new();
        let http = NativeHttp::default();
        let script = Script {
            source: script.to_string(),
            timeout: None,
            ..Default::default()
        };
        Worker::new(script, &v8, http)
    }

    #[tokio::test]
    async fn test_ok_response() {
        let script = r#"
            function onEvent(request) {
                return {status: 200}
            }
        "#;
        let worker = new_worker(script).unwrap();
        let request = reqwest::Request::new(
            reqwest::Method::GET,
            Url::parse("http://jsonplaceholder.typicode.com/users/1").unwrap(),
        );
        let response = worker.on_event(request).await.unwrap();
        assert_eq!(response.status.as_u16(), 200);
    }

    #[tokio::test]
    async fn test_url() {
        let script = r#"
            function onEvent(request) {
                return {body: {url: request.url}}
            }
        "#;
        let worker = new_worker(script).unwrap();
        let request = reqwest::Request::new(
            reqwest::Method::GET,
            Url::parse("http://jsonplaceholder.typicode.com/users/1").unwrap(),
        );
        let response = worker.on_event(request).await.unwrap();
        let body = String::from_utf8(response.body.to_vec()).unwrap();
        assert_eq!(response.status.as_u16(), 200);
        assert_eq!(
            body,
            r#"{"url":"http://jsonplaceholder.typicode.com/users/1"}"#
        );
    }
}
