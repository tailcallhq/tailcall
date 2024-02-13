use std::sync::Arc;

use anyhow::{anyhow, Result};
use hyper::{Body, Request, Response};

use crate::app_context::AppContext;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::blueprint::{validate_operations, Blueprint, OperationQuery};
use crate::config::reader::ConfigReader;
use crate::config::{Config, ConfigModule, Source};
use crate::fmt::Fmt;
use crate::http::handle_request;
use crate::print_schema;
use crate::runtime::TargetRuntime;
use crate::valid::{ValidationError, Validator};

#[derive(Clone)]
pub struct TailcallBuilder {
    runtime: TargetRuntime,
}
#[derive(Clone)]
pub struct TailcallExecutor {
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
    ) -> Result<TailcallExecutor> {
        let reader = ConfigReader::init(self.runtime.clone());
        let config = Config::from_source(source, &schema.to_string())?;

        let config_module = reader
            .resolve(config, relative_path)
            .await?;
        let blueprint = Blueprint::try_from(&config_module.clone())?;
        let app_ctx = AppContext::new(blueprint, self.runtime);
        let app_ctx = Arc::new(app_ctx);
        Ok(TailcallExecutor { config_module, app_ctx })
    }
    pub async fn with_config_paths<T: ToString>(self, files: &[T]) -> Result<TailcallExecutor> {
        let reader = ConfigReader::init(self.runtime.clone());
        let config_module = reader.read_all(files).await?;
        let blueprint = Blueprint::try_from(&config_module)?;
        let app_ctx = AppContext::new(blueprint, self.runtime);
        let app_ctx = Arc::new(app_ctx);
        Ok(TailcallExecutor { config_module, app_ctx })
    }
}

impl TailcallExecutor {
    pub async fn validate(
        &self,
        n_plus_one_queries: bool,
        schema: bool,
        ops: Vec<OperationQuery>,
    ) -> Result<String> {
        log::info!("{}", "Config successfully validated".to_string());

        let mut result_str = display_config(&self.config_module, n_plus_one_queries);
        if schema {
            let tbp = display_schema(&self.app_ctx.blueprint);
            result_str = format!("{result_str}\n{tbp}");
        }

        validate_operations(&self.app_ctx.blueprint, ops)
            .await
            .to_result()?;

        Ok(result_str)
    }
    pub async fn execute(&self, req: Request<Body>) -> Result<Response<Body>> {
        if self.app_ctx.blueprint.server.enable_batch_requests {
            handle_request::<GraphQLBatchRequest>(req, self.app_ctx.clone()).await
        } else {
            handle_request::<GraphQLRequest>(req, self.app_ctx.clone()).await
        }
    }
}

fn display_config(config: &Config, n_plus_one_queries: bool) -> String {
    let seq = vec![Fmt::n_plus_one_data(n_plus_one_queries, config)];
    Fmt::table(seq)
}
pub fn display_schema(blueprint: &Blueprint) -> String {
    let p1 = Fmt::heading(&"GraphQL Schema:\n".to_string());
    let sdl = blueprint.to_schema();
    format!("{p1}\n{}\n", print_schema::print_schema(sdl))
}
