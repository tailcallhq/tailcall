use std::sync::Arc;

use tailcall::{AppContext, Blueprint, ConfigReader, EndpointSet, TargetRuntime};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::env::WasmEnv;
use crate::js_val::JsVal;
use crate::runtime::init_rt;
use crate::TailcallExecutor;

#[wasm_bindgen]
struct TailcallBuilder {
    target_runtime: TargetRuntime,
    env: WasmEnv, // TODO
    configs: Vec<String>,
}

#[wasm_bindgen]
impl TailcallBuilder {
    #[wasm_bindgen(constructor)]
    pub fn init() -> Self {
        Self {
            target_runtime: init_rt(),
            env: WasmEnv::init(),
            configs: vec![],
        }
    }

    pub async fn with_file(
        self,
        path: String,
        content: String,
    ) -> Result<TailcallBuilder, JsValue> {
        self.with_file_inner(path, content)
            .await
            .map_err(|e| JsValue::from(e.to_string()))
    }
    async fn with_file_inner<T: AsRef<[u8]>>(
        self,
        path: String,
        content: T,
    ) -> anyhow::Result<TailcallBuilder> {
        self.target_runtime
            .file
            .write(&path, content.as_ref())
            .await?;
        Ok(self)
    }

    pub async fn with_config(
        self,
        path: String,
        content: String,
    ) -> Result<TailcallBuilder, JsValue> {
        self.with_config_inner(path, content)
            .await
            .map_err(|e| JsValue::from(e.to_string()))
    }

    async fn with_config_inner(
        mut self,
        path: String,
        content: String,
    ) -> anyhow::Result<TailcallBuilder> {
        if url::Url::parse(&content).is_ok() {
            self.configs.push(content);
        } else {
            self.target_runtime
                .file
                .write(&path, content.as_bytes())
                .await?;
            self.configs.push(path);
        }
        Ok(self)
    }

    pub fn with_env(self, key: String, val: String) -> TailcallBuilder {
        self.env.set(key, val);
        self
    }
    pub async fn build(self) -> Result<TailcallExecutor, JsValue> {
        match self.build_inner().await {
            Ok(v) => Ok(v),
            Err(e) => Err(JsVal::from(e).into()),
        }
    }
    async fn build_inner(mut self) -> anyhow::Result<TailcallExecutor> {
        self.target_runtime.env = Arc::new(self.env);

        let reader = ConfigReader::init(self.target_runtime.clone());
        let config_module = reader.read_all(&self.configs).await?;

        let blueprint = Blueprint::try_from(&config_module)?;
        let app_context = Arc::new(AppContext::new(
            blueprint,
            self.target_runtime,
            EndpointSet::default(),
        ));

        Ok(TailcallExecutor { app_context })
    }
}
