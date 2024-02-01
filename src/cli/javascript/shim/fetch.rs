use hyper::body::Bytes;
use mini_v8::{Function, Invocation, MiniV8, Values};
use serde_json::json;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::spawn_local;

use crate::channel::{JsRequest, JsResponse};
use crate::cli::javascript::async_wrapper::FetchMessage;
use crate::cli::javascript::async_wrapper::FetchResult;
use crate::cli::javascript::serde_v8::SerdeV8;
use crate::http::Response;
use crate::ToAnyHow;

pub const FETCH: &str = "__fetch__";

pub fn init(v8: MiniV8, http_sender: mpsc::UnboundedSender<FetchMessage>) -> anyhow::Result<()> {
    let mv8 = v8.clone();
    let fetch = v8.create_function(move |invocation| {
        let args = JSFetchArgs::try_from(&mv8, &invocation).map_err(|_| {
            mini_v8::Error::ToJsConversionError { from: "MiniV8 Invocation", to: "JSFetchArgs" }
        })?;

        spawn_local(fetch(mv8.clone(), http_sender.clone(), args));
        Ok(mini_v8::Value::Undefined)
    });
    v8.global()
        .set(FETCH, fetch)
        .or_anyhow(format!("Could not set {} in global v8 object", FETCH).as_str())?;

    Ok(())
}

#[derive(Clone)]
struct JSFetchArgs {
    request: JsRequest,
    callback: Function,
}

impl JSFetchArgs {
    fn try_from(v8: &MiniV8, value: &Invocation) -> anyhow::Result<Self> {
        let request = JsRequest::from_v8(&value.args.get(0))?;

        let callback = value.args.get(1).as_function().cloned();
        let callback = callback.ok_or(anyhow::anyhow!(
            "Second argument to fetch must be a function"
        ))?;

        Ok(Self { request, callback })
    }
}

async fn fetch(
    v8: MiniV8,
    http_sender: mpsc::UnboundedSender<FetchMessage>,
    args: JSFetchArgs,
) -> anyhow::Result<()> {
    let (tx, rx) = oneshot::channel::<FetchResult>();

    http_sender.send((tx, args.request))?;

    let response = rx.await?;
    match response {
        Ok(response) => {
            args.callback
                .call(Values::from_iter(vec![
                    mini_v8::Value::Null,
                    response.to_v8(&v8)?,
                ]))
                .or_anyhow("failed to call callback")?;

            Ok(())
        }
        Err(e) => {
            let error = e.to_string();
            let error = error.clone();
            args.callback
                .call(Values::from_iter(vec![
                    mini_v8::Value::String(v8.create_string(error.as_str())),
                    mini_v8::Value::Null,
                ]))
                .or_anyhow("failed to call callback")?;
            Ok(())
        }
    }
}
