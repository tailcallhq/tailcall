use std::sync::Arc;

use tailcall::core::app_context::AppContext;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::reader::ConfigReader;
use tailcall::core::config::ConfigModule;
use tailcall::core::merge_right::MergeRight;
use tailcall::core::rest::EndpointSet;
use tailcall::core::runtime::TargetRuntime;
use tailcall::core::variance::Invariant;
use tailcall_valid::Validator;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::env::WasmEnv;
use crate::runtime::init_rt;
use crate::{to_val, TailcallExecutor};

#[wasm_bindgen]
pub struct TailcallBuilder {
    reader: ConfigReader,
    rt: TargetRuntime,
    env: WasmEnv,
    module: ConfigModule,
}

#[wasm_bindgen]
impl TailcallBuilder {
    #[wasm_bindgen(constructor)]
    pub fn init() -> Self {
        Self::init_inner(init_rt())
    }

    fn init_inner(rt: TargetRuntime) -> Self {
        let reader = ConfigReader::init(rt.clone());
        Self { rt, reader, module: Default::default(), env: WasmEnv::init() }
    }

    pub async fn with_config(self, path: String) -> Result<TailcallBuilder, JsValue> {
        self.with_config_inner(path).await.map_err(to_val)
    }

    async fn with_config_inner(mut self, url: String) -> anyhow::Result<TailcallBuilder> {
        if url::Url::parse(&url).is_ok() {
            self.module = self
                .module
                .unify(self.reader.read(url).await?)
                .to_result()?;
        } else {
            return Err(anyhow::anyhow!("Config can only be loaded over URL"));
        }
        Ok(self)
    }

    pub fn with_env(self, key: String, val: String) -> TailcallBuilder {
        self.env.set(key, val);
        self
    }
    pub async fn build(self) -> Result<TailcallExecutor, JsValue> {
        self.build_inner().await.map_err(to_val)
    }
    async fn build_inner(mut self) -> anyhow::Result<TailcallExecutor> {
        self.rt.env = Arc::new(self.env);

        let blueprint = Blueprint::try_from(&self.module)?;
        let app_context = Arc::new(AppContext::new(blueprint, self.rt, EndpointSet::default()));

        Ok(TailcallExecutor { app_context })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use anyhow::anyhow;
    use hyper::body::Bytes;
    use reqwest::Request;
    use serde_json::{json, Value};
    use tailcall::core::http::Response;
    use tailcall::core::HttpIO;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::runtime::init_rt;

    struct MockHttp {}

    const CONFIG: &str = r#"
        schema @server(port: 8000) {
          query: Query
        }

        type Query {
          hello: String! @expr(body: "Alo")
        }
    "#;

    #[async_trait::async_trait]
    impl HttpIO for MockHttp {
        async fn execute(&self, request: Request) -> anyhow::Result<Response<Bytes>> {
            let resp = tailcall::core::http::Response::empty();
            match request.url().path() {
                "/hello.graphql" => Ok(resp.body(Bytes::from(CONFIG))),
                _ => Ok(resp),
            }
        }
    }

    #[wasm_bindgen_test]
    async fn test() {
        crate::start();
        let mut rt = init_rt();
        rt.http = Arc::new(MockHttp {});
        let builder = super::TailcallBuilder::init_inner(rt);
        let executor = builder
            .with_config("http://fake.host/hello.graphql".to_string())
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
