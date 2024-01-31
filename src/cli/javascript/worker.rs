use hyper::body::Bytes;
use mini_v8::{Function, MiniV8, Values};
use tokio::sync::{mpsc, oneshot};

use super::async_wrapper::ChannelMessage;
use super::serde_v8::SerdeV8;
use crate::channel::{JsRequest, JsResponse};
use crate::http::Response;
use crate::{blueprint, ToAnyHow};

#[derive(Clone)]
pub struct Worker {
    v8: MiniV8,
    on_event_js: Function,
}

fn create_closure(script: &str) -> String {
    format!("(function() {{{} return onEvent}})();", script)
}

impl Worker {
    pub fn new(
        script: blueprint::Script,
        http_sender: mpsc::UnboundedSender<ChannelMessage>,
    ) -> anyhow::Result<Self> {
        let v8 = MiniV8::new();
        super::shim::init(&v8, http_sender)?;
        let script = mini_v8::Script {
            source: create_closure(script.source.as_str()),
            timeout: script.timeout,
            ..Default::default()
        };
        let value: mini_v8::Value = v8
            .eval(script)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let on_event_js = value
            .as_function()
            .ok_or_else(|| anyhow::anyhow!("expected an 'onEvent' function"))?
            .clone();

        Ok(Self { v8, on_event_js })
    }

    pub async fn on_event(&self, request: reqwest::Request) -> anyhow::Result<Response<Bytes>> {
        let (tx, rx) = oneshot::channel::<serde_json::Value>();
        let mut tx = Some(tx);
        let js_request = JsRequest::try_from(&request)?;
        let on_event_js = self.on_event_js.clone();

        let js_request_v8 = js_request.to_v8(&self.v8)?;

        let cb: mini_v8::Value = mini_v8::Value::Function(self.v8.create_function_mut({
            move |invocation| {
                let Some(tx) = tx.take() else {
                    return Err(mini_v8::Error::ExternalError(
                        "Multiple callback calls".into(),
                    ));
                };

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

        on_event_js
            .call(args)
            .or_anyhow("failed to dispatch request to js-worker: ")?;

        let result = rx
            .await
            .or_anyhow("failed to receive response from js-worker")?;
        // Check if the result is a response

        // TODO: simplify conversions
        let js_response: JsResponse = serde_json::from_value(result)?;
        let response = Response::<Bytes>::try_from(js_response)?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use hyper::body::Buf;
    use pretty_assertions::assert_eq;
    use serde_json::{json, Value};
    use tokio::spawn;
    use url::Url;

    use super::*;
    use crate::blueprint::Script;
    use crate::cli::javascript::shim::fetch::FETCH;

    async fn new_worker(script: &str) -> anyhow::Result<Worker> {
        let script = Script {
            source: script.to_string(),
            timeout: None,
            ..Default::default()
        };
        let (http_sender, mut http_receiver) = mpsc::unbounded_channel::<ChannelMessage>();

        spawn(async move {
            while let Some((respond, _request)) = http_receiver.recv().await {
                let _ = respond.send(Ok(Response::default()));
            }
        });

        Worker::new(script, http_sender)
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
