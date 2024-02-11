use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use hyper::{Body, Request, Response};

use crate::app_context::AppContext;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::blueprint::Blueprint;
use crate::config::reader::ConfigReader;
use crate::config::{Config, Source};
use crate::http::handle_request;
use crate::runtime::TargetRuntime;

pub struct TailcallBuilder {
    runtime: TargetRuntime,
}

pub struct Tailcall {
    app_ctx: Arc<AppContext>,
}

impl TailcallBuilder {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { runtime }
    }
    pub async fn with_config(self, source: Source, schema: &str) -> Result<Tailcall> {
        let reader = ConfigReader::init(self.runtime.clone());
        let config = Config::from_source(source, schema)?;
        let config_module = reader.resolve(config, None).await?;
        let blueprint = Blueprint::try_from(&config_module)?;
        let app_ctx = AppContext::new(blueprint, self.runtime);
        let app_ctx = Arc::new(app_ctx);
        Ok(Tailcall { app_ctx })
    }
    pub async fn with_config_paths<T: AsRef<Path>>(self, files: &[T]) -> Result<Tailcall> {
        let reader = ConfigReader::init(self.runtime.clone());
        let config_module = reader.read_all(files).await;
        let blueprint = Blueprint::try_from(&config_module)?;
        let app_ctx = AppContext::new(blueprint, self.runtime);
        let app_ctx = Arc::new(app_ctx);
        Ok(Tailcall { app_ctx })
    }
}

impl Tailcall {
    pub async fn execute(&self, req: Request<Body>) -> Result<Response<Body>> {
        handle_request::<GraphQLRequest>(req, self.app_ctx.clone()).await
    }

    pub async fn execute_batch(&self, req: Request<Body>) -> Result<Response<Body>> {
        handle_request::<GraphQLBatchRequest>(req, self.app_ctx.clone()).await
    }
}
