#![allow(unused)]

use std::fmt::Display;
use std::panic;
use std::sync::Arc;

use serde_json::json;
use tailcall::{handle_request, AppContext, GraphQLRequest};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

mod builder;
mod env;
mod file;
mod http;
mod runtime;

#[wasm_bindgen]
pub struct TailcallExecutor {
    app_context: Arc<AppContext>,
}

#[wasm_bindgen]
impl TailcallExecutor {
    pub async fn execute(&self, query: String) -> Result<JsValue, JsValue> {
        self.execute_inner(query).await.map(to_val).map_err(to_val)
    }
    async fn execute_inner(&self, query: String) -> anyhow::Result<String> {
        let body = json!({"query":query}).to_string();
        let req =
            hyper::Request::post("http://fake.host/graphql").body(hyper::body::Body::from(body))?;

        let resp = handle_request::<GraphQLRequest>(req, self.app_context.clone()).await?;
        tracing::debug!("{:#?}", resp);

        let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
        let body_str = String::from_utf8(body_bytes.to_vec())?;
        Ok(body_str)
    }
}

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();

    tracing_subscriber::fmt()
        .with_writer(
            // To avoid trace events in the browser from showing their JS backtrace
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();
}

pub fn to_val<T: Display>(val: T) -> JsValue {
    JsValue::from(val.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::anyhow;
    use serde_json::{json, Value};
    use wasm_bindgen_test::wasm_bindgen_test;

    const CONFIG: &str = r#"
        schema @server(port: 8000) {
          query: Query
        }

        type Query {
          hello: String! @expr(body: "Alo")
        }
    "#;

    #[wasm_bindgen_test]
    async fn test() {
        super::start();
        let executor = super::builder::TailcallBuilder::init()
            .with_config("hello.graphql".to_string(), CONFIG.to_string())
            .await
            .unwrap()
            .build()
            .await
            .unwrap();
        let response = executor
            .execute("query { hello }".to_string())
            .await
            .unwrap();
        let value: Value = serde_json::from_str(&response.as_string().unwrap()).unwrap();
        assert_eq!(value, json!({"data": {"hello": "Alo"}}));
    }
}
