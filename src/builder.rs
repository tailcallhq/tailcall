use std::sync::Arc;

use anyhow::Result;
use hyper::{Body, Request, Response};

use crate::app_context::AppContext;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::blueprint::Blueprint;
use crate::config::reader::ConfigReader;
use crate::config::{Config, ConfigModule, Source};
use crate::http::handle_request;
use crate::runtime::TargetRuntime;

#[derive(Clone)]
pub struct TailcallBuilder {
    runtime: TargetRuntime,
}
#[derive(Clone)]
pub struct Tailcall {
    pub config_module: ConfigModule,
    pub app_ctx: Arc<AppContext>,
}

impl TailcallBuilder {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { runtime }
    }
    pub async fn with_config<T: ToString>(
        self,
        source: Source,
        schema: T,
        relative_path: Option<String>,
    ) -> Result<Tailcall> {
        let reader = ConfigReader::init(self.runtime.clone());
        let config = Config::from_source(source, &schema.to_string())?;
        let config_module = reader.resolve(config, relative_path).await?;
        let blueprint = Blueprint::try_from(&config_module.clone())?;
        let app_ctx = AppContext::new(blueprint, self.runtime);
        let app_ctx = Arc::new(app_ctx);
        Ok(Tailcall { config_module, app_ctx })
    }
    pub async fn with_config_paths<T: ToString>(self, files: &[T]) -> Result<Tailcall> {
        let reader = ConfigReader::init(self.runtime.clone());
        let config_module = reader.read_all(files).await?;
        let blueprint = Blueprint::try_from(&config_module.clone())?;
        let app_ctx = AppContext::new(blueprint, self.runtime);
        let app_ctx = Arc::new(app_ctx);
        Ok(Tailcall { config_module, app_ctx })
    }
}

impl Tailcall {
    pub async fn execute(&self, req: Request<Body>) -> Result<Response<Body>> {
        if self.app_ctx.blueprint.server.enable_batch_requests {
            handle_request::<GraphQLBatchRequest>(req, self.app_ctx.clone()).await
        } else {
            handle_request::<GraphQLRequest>(req, self.app_ctx.clone()).await
        }
    }
}
