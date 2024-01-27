#![allow(unused_variables)]

mod cache;
mod env;
mod http;

use std::fmt::Display;
use std::sync::Arc;

use serde_json::json;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::{Config, Source};
use tailcall::http::{handle_request, AppContext};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::cache::WasmCache;
use crate::env::WasmEnv;
use crate::http::WasmHttp;

#[wasm_bindgen]
struct GraphQLExecutor {
    app_ctx: Arc<AppContext>,
}
#[wasm_bindgen]
impl GraphQLExecutor {
    #[wasm_bindgen(constructor)]
    pub fn new(schema: String, mut source: String) -> Result<GraphQLExecutor, JsValue> {
        if !source.starts_with(".") {
            source = format!(".{source}");
        }
        let executor = Self::get_app_ctx(schema, source).map_err(to_jsvalue)?;
        Ok(executor)
    }
    #[wasm_bindgen]
    pub async fn execute(&self, query: String) -> Result<JsValue, JsValue> {
        let body = json!({"query":query}).to_string();
        let req = hyper::Request::post("http://fake.host/graphql")
            .body(hyper::body::Body::from(body))
            .map_err(to_jsvalue)?;
        let resp = handle_request::<GraphQLRequest>(req, self.app_ctx.clone())
            .await
            .map_err(to_jsvalue)?;
        log::debug!("{:#?}", resp);
        let body_bytes = hyper::body::to_bytes(resp.into_body())
            .await
            .map_err(to_jsvalue)?;
        Ok(to_jsvalue(
            String::from_utf8(body_bytes.to_vec()).map_err(to_jsvalue)?,
        ))
    }
    fn get_app_ctx(schema: String, source: String) -> anyhow::Result<GraphQLExecutor> {
        let source = Source::detect(source.as_str())?;
        let config = Config::from_source(source, schema.as_str())?;
        let blueprint = Blueprint::try_from(&config)?;
        let http_io = Arc::new(WasmHttp::new());
        let http_clone = http_io.clone();
        let app_ctx = Arc::new(AppContext::new(
            blueprint,
            http_io,
            http_clone,
            Arc::new(WasmEnv::new()),
            Arc::new(WasmCache::init()),
            None,
        ));
        Ok(GraphQLExecutor { app_ctx })
    }
}

fn to_jsvalue<T: Display>(val: T) -> JsValue {
    JsValue::from_str(val.to_string().as_str())
}

#[wasm_bindgen(start)]
fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    console_error_panic_hook::set_once();
}

/*fn main() {
    extern crate cfg_if;
    extern crate wasm_bindgen;

    use cfg_if::cfg_if;
    use wasm_bindgen::prelude::*;

    cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

    #[wasm_bindgen]
    extern {
        fn alert(s: &str);
    }

    #[wasm_bindgen]
    pub fn greet() {
        alert("Hello, wasm-game-of-life!");
    }

}*/
