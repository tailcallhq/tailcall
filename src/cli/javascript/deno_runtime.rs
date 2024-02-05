use std::{
    cell::{OnceCell, RefCell},
    collections::BTreeMap,
    thread,
};

use deno_core::{v8, FastString, JsRuntime};
use hyper::{
    body::Bytes,
    header::{HeaderName, HeaderValue},
};
use reqwest::Request;
use serde::{Deserialize, Serialize};

use crate::{blueprint, http::Response, is_default, WorkerIO};

struct LocalRuntime {
    value: v8::Global<v8::Value>,
    js_runtime: JsRuntime,
}

thread_local! {
  static LOCAL_RUNTIME: RefCell<OnceCell<LocalRuntime>> = RefCell::new(OnceCell::new());
}

#[derive(Serialize, Deserialize)]
struct Message {
    pub message: MessageContent,
    pub id: Option<u64>,
}

#[derive(Serialize, Deserialize)]
enum MessageContent {
    Request(JsRequest),
    Response(JsResponse),
    Empty,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsRequest {
    url: String,
    method: String,
    #[serde(skip_serializing_if = "is_default")]
    headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "is_default")]
    body: Option<Bytes>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsResponse {
    status: u16,
    #[serde(skip_serializing_if = "is_default")]
    headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "is_default")]
    body: Option<Bytes>,
}

impl LocalRuntime {
    fn try_new(script: blueprint::Script) -> anyhow::Result<Self> {
        let source = create_closure(script.source.as_str());
        let mut js_runtime = JsRuntime::new(Default::default());
        let value = js_runtime.execute_script("<anon>", FastString::from(source))?;
        log::debug!("JS Runtime created: {:?}", thread::current().name());
        Ok(Self { value, js_runtime })
    }
}

fn create_closure(script: &str) -> String {
    format!("(function() {{{} return onEvent}})();", script)
}

pub struct Runtime {
    script: blueprint::Script,
}

impl Runtime {
    pub fn new(script: blueprint::Script) -> Self {
        Self { script: script }
    }
}

#[async_trait::async_trait]
impl WorkerIO<Message, Message> for Runtime {
    async fn dispatch(&self, event: Message) -> anyhow::Result<Message> {
        LOCAL_RUNTIME.with(move |cell| {
            let script = self.script.clone();
            cell.borrow()
                .get_or_init(|| LocalRuntime::try_new(script).unwrap());
            on_event(event)
        })
    }
}

fn on_event(message: Message) -> anyhow::Result<Message> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let local_runtime = cell.get_mut().unwrap();
        let scope = &mut local_runtime.js_runtime.handle_scope();
        let value = &local_runtime.value;
        let local_value = v8::Local::new(scope, value);
        let closure: v8::Local<v8::Function> = local_value.try_into()?;
        let input = serde_v8::to_v8(scope, message)?;
        let null_ctx = v8::null(scope);
        let output = closure.call(scope, null_ctx.into(), &[input]);

        match output {
            None => Ok(Message { message: MessageContent::Empty, id: None }),
            Some(output) => Ok(serde_v8::from_v8(scope, output)?),
        }
    })
}

// Response implementations
impl TryFrom<JsResponse> for Response<Bytes> {
    type Error = anyhow::Error;

    fn try_from(res: JsResponse) -> Result<Self, Self::Error> {
        let status = reqwest::StatusCode::from_u16(res.status as u16)?;
        let headers = create_header_map(res.headers)?;
        let body = serde_json::to_string(&res.body)?;
        Ok(Response { status, headers, body: Bytes::from(body) })
    }
}

impl TryFrom<&Response<Bytes>> for JsResponse {
    type Error = anyhow::Error;

    fn try_from(res: &Response<Bytes>) -> Result<Self, Self::Error> {
        let status = res.status.as_u16();
        let mut headers = BTreeMap::new();
        for (key, value) in res.headers.iter() {
            let key = key.to_string();
            let value = value.to_str()?.to_string();
            headers.insert(key, value);
        }

        let body = serde_json::from_slice(res.body.as_ref())?;
        Ok(JsResponse { status, headers, body })
    }
}

// Request implementations
impl TryFrom<JsRequest> for reqwest::Request {
    type Error = anyhow::Error;

    fn try_from(req: JsRequest) -> Result<Self, Self::Error> {
        let mut request = reqwest::Request::new(
            reqwest::Method::from_bytes(req.method.as_bytes())?,
            req.url.parse()?,
        );
        let headers = create_header_map(req.headers)?;
        request.headers_mut().extend(headers);
        let body = serde_json::to_string(&req.body)?;
        let _ = request.body_mut().insert(reqwest::Body::from(body));
        Ok(request)
    }
}

impl TryFrom<&reqwest::Request> for JsRequest {
    type Error = anyhow::Error;

    fn try_from(req: &Request) -> Result<Self, Self::Error> {
        let url = req.url().to_string();
        let method = req.method().as_str().to_string();
        let headers = req
            .headers()
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect::<BTreeMap<String, String>>();
        let body = req
            .body()
            .and_then(|body| body.as_bytes())
            .and_then(|body| serde_json::from_slice(body).ok());
        Ok(JsRequest { url, method, headers, body })
    }
}

fn create_header_map(
    headers: BTreeMap<String, String>,
) -> anyhow::Result<reqwest::header::HeaderMap> {
    let mut header_map = reqwest::header::HeaderMap::new();
    for (key, value) in headers.iter() {
        let key = HeaderName::from_bytes(key.as_bytes())?;
        let value = HeaderValue::from_str(value.as_str())?;
        header_map.insert(key, value);
    }
    Ok(header_map)
}
