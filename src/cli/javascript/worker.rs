use std::sync::Arc;

use hyper::body::Bytes;
use mini_v8::Values;

use super::serde_v8::SerdeV8;
use super::sync_v8::{SyncV8, SyncV8Function};
use crate::channel::{JsRequest, JsResponse};
use crate::http::Response;
use crate::{blueprint, HttpIO, ToAnyHow};

pub struct Worker {
    sync_v8: SyncV8,
    on_event_js: SyncV8Function,
}

fn create_closure(script: &str) -> String {
    format!("(function() {{{} return onEvent}})();", script)
}

impl Worker {
    pub async fn new(script: blueprint::Script, http: impl HttpIO) -> anyhow::Result<Self> {
        let sync_v8 = SyncV8::new();
        super::shim::init(&sync_v8, Arc::new(http)).await?;
        let v8 = sync_v8.clone();
        let closure = sync_v8
            .clone()
            .borrow_ret(move |mv8| {
                let script = mini_v8::Script {
                    source: create_closure(script.source.as_str()),
                    timeout: script.timeout,
                    ..Default::default()
                };
                let value: mini_v8::Value = mv8
                    .eval(script)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let closure = value
                    .as_function()
                    .ok_or_else(|| anyhow::anyhow!("expected an 'onEvent' function"))?
                    .clone();

                Ok(v8.as_sync_function(closure))
            })
            .await?;
        Ok(Self { sync_v8: sync_v8.clone(), on_event_js: closure })
    }

    pub async fn on_event(&self, request: reqwest::Request) -> anyhow::Result<Response<Bytes>> {
        let (tx, mut rx) = tokio::sync::broadcast::channel::<serde_json::Value>(1024);
        let sync_v8 = self.sync_v8.clone();
        let js_request = JsRequest::try_from(&request)?;
        let on_event_js = self.on_event_js.clone();

        sync_v8
            .borrow(move |mv8| {
                let js_request_v8 = js_request.to_v8(mv8)?;

                let cb: mini_v8::Value = mini_v8::Value::Function(mv8.create_function({
                    move |invocation| {
                        // FIXME: get arg.get(0) as error
                        let response = invocation.args.get(1);
                        let json = serde_json::Value::from_v8(&response).unwrap();
                        tx.send(json).map_err(|e| {
                            mini_v8::Error::ExternalError(anyhow::anyhow!(e.to_string()).into())
                        })?;
                        Ok(mini_v8::Value::Undefined)
                    }
                }));

                // Initiate an async call
                let args: Values = Values::from_iter(vec![js_request_v8, cb]);

                // NOTE: This doesn't complete
                let _err = on_event_js
                    .call::<()>(args)
                    .or_anyhow("failed to dispatch request to js-worker: ");

                Ok(())
            })
            .await?;

        let result = rx
            .recv()
            .await
            .or_anyhow("failed to receive response from js-worker")?;
        // Check if the result is a response

        let js_response: JsResponse = serde_json::from_value(result)?;
        let response = Response::<Bytes>::try_from(js_response)?;
        Ok(response)
    }
}

#[async_trait::async_trait]
impl HttpIO for Worker {
    async fn execute(
        &self,
        request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>> {
        self.on_event(request).await
    }
}

#[cfg(test)]
mod test {
    use hyper::body::Buf;
    use pretty_assertions::assert_eq;
    use serde_json::{json, Value};
    use url::Url;

    use super::*;
    use crate::blueprint::Script;
    use crate::cli::javascript::shim::fetch::FETCH;
    use crate::cli::NativeHttp;

    async fn new_worker(script: &str) -> anyhow::Result<Worker> {
        let http = NativeHttp::default();
        let script = Script {
            source: script.to_string(),
            timeout: None,
            ..Default::default()
        };
        Worker::new(script, http).await
    }

    fn new_get_request(url: &str) -> anyhow::Result<reqwest::Request> {
        Ok(reqwest::Request::new(
            reqwest::Method::GET,
            Url::parse(url)?,
        ))
    }

    #[tokio::test]
    async fn test_ok_response() {
        let script = r#"
            function onEvent(request, cb) {
                return cb(null, {status: 200})
            }
        "#;
        let worker = new_worker(script).await.unwrap();
        let request = new_get_request("https://jsonplaceholder.typicode.com/users/1").unwrap();
        let response = worker.on_event(request).await.unwrap();
        assert_eq!(response.status.as_u16(), 200);
    }

    #[tokio::test]
    async fn test_url() {
        let script = r#"
            function onEvent(request, cb) {
                return cb(null, {body: {url: request.url}})
            }
        "#;
        let worker = new_worker(script).await.unwrap();
        let request = new_get_request("https://jsonplaceholder.typicode.com/users/1").unwrap();
        let response = worker.on_event(request).await.unwrap();
        let body = String::from_utf8(response.body.to_vec()).unwrap();
        assert_eq!(response.status.as_u16(), 200);
        assert_eq!(
            body,
            r#"{"url":"https://jsonplaceholder.typicode.com/users/1"}"#
        );
    }

    #[tokio::test]
    async fn test_fetch() {
        let script = format!(
            r#"
            function onEvent(request, cb) {{
                return {} (request, (err, response) => {{
                    cb(null, response)
                }})
            }}
        "#,
            FETCH
        );

        let worker = new_worker(script.as_str()).await.unwrap();
        let request = new_get_request("https://jsonplaceholder.typicode.com/posts/1").unwrap();
        let response = worker.on_event(request).await.unwrap();

        assert_eq!(response.status.as_u16(), 200);

        let actual = serde_json::from_slice::<Value>(response.body.chunk()).unwrap();
        let expected = json!({
            "body": "quia et suscipit\nsuscipit recusandae consequuntur expedita et cum\nreprehenderit molestiae ut ut quas totam\nnostrum rerum est autem sunt rem eveniet architecto",
            "id": 1.0,
            "title": "sunt aut facere repellat provident occaecati excepturi optio reprehenderit",
            "userId": 1.0,
        });
        assert_eq!(actual, expected);
    }
}
