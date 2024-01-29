use mini_v8::{Invocation, Value, Values};

use crate::cli::javascript::serde_v8::SerdeV8;
use crate::cli::javascript::sync_v8::SyncV8;
use crate::ToAnyHow;

pub const FETCH: &str = "__tailcall__fetch__";
pub async fn init(v8: &SyncV8) -> anyhow::Result<()> {
    v8.borrow(|v8| {
        let fetch = v8.create_function(fetch);
        v8.global()
            .set(FETCH, fetch)
            .or_anyhow(format!("Could not set {} in global v8 object", FETCH).as_str())?;

        Ok(())
    })
    .await
}

fn fake_http_response() -> serde_json::Value {
    serde_json::json!({
        "status": 200,
        "headers": {
            "content-type": "application/json",
        },
        "body": {
            "a": "1",
            "b": "2",
            "c": "3",
        }
    })
}

struct JSFetchInvocation {
    url: String,
    callback: Box<dyn FnOnce(serde_json::Value, serde_json::Value)>,
}

impl JSFetchInvocation {
    fn try_from(value: &Invocation) -> anyhow::Result<Self> {
        let v8 = &value.mv8;
        let url = value.args.get(0);
        let url = url.as_string().ok_or(anyhow::anyhow!(
            "First argument to fetch must be a string, got {:?}",
            url
        ))?;

        let callback = value.args.get(1).as_function().cloned();
        let callback = callback.ok_or(anyhow::anyhow!(
            "Second argument to fetch must be a function"
        ))?;

        let url = url.to_string();
        let v8 = v8.clone();
        Ok(Self {
            url,
            callback: Box::new(
                move |error: serde_json::Value, response: serde_json::Value| {
                    if !error.is_null() {
                        let error = error.to_v8(&v8).unwrap();
                        callback
                            .call::<Values, ()>(Values::from_iter(vec![error]))
                            .unwrap();
                        return;
                    }

                    let response = response.to_v8(&v8).unwrap();
                    callback
                        .call::<Values, ()>(Values::from_iter(vec![Value::Null, response]))
                        .unwrap();
                },
            ),
        })
    }
}

fn fetch(invocation: mini_v8::Invocation) -> Result<mini_v8::Value, mini_v8::Error> {
    let invocation = JSFetchInvocation::try_from(&invocation).unwrap();
    let response = fake_http_response();
    (invocation.callback)(serde_json::Value::Null, response);
    Ok(mini_v8::Value::Undefined)
}
