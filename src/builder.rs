use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use hyper::{Body, Request, Response};

use crate::app_context::AppContext;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::blueprint::{validate_operations, Blueprint, OperationQuery};
use crate::config::{Config, ConfigModule, ConfigReader, Source};
use crate::fmt::Fmt;
use crate::http::handle_request;
use crate::print_schema;
use crate::runtime::TargetRuntime;
use crate::valid::Validator;

/// Struct to build TailcallExecutor
#[derive(Clone)]
pub struct TailcallBuilder {
    /// Holds a list of file paths
    files: Vec<String>,
    /// Holds a list of schema information
    schemas: Vec<SchemaHolder>,
}

/// Struct to hold schema information while building executor
#[derive(Clone)]
struct SchemaHolder {
    /// Holds a type of schema
    source: Source,
    /// Holds content of schema
    schema: String,
    /// Holds path to parent dir for content in @link
    parent_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub struct TailcallExecutor {
    pub config_module: ConfigModule,
    pub app_ctx: Arc<AppContext>,
}

impl TailcallBuilder {
    /// Creates a new instance of TailcallBuilder
    pub fn new() -> Self {
        Self { files: vec![], schemas: vec![] }
    }

    /// Adds a configuration source with schema and optional parent directory
    pub fn with_config_source<T: ToString, P: AsRef<Path>>(
        mut self,
        source: Source,
        schema: T,
        parent_dir: Option<P>,
    ) -> Self {
        let parent_dir = parent_dir.map(|p| p.as_ref().into());
        self.schemas
            .push(SchemaHolder { source, schema: schema.to_string(), parent_dir });
        self
    }

    /// Adds an array of paths to configuration files
    /// The file IO is carried out using runtime passed in build function
    pub fn with_config_files<T: ToString>(mut self, files: &[T]) -> Self {
        self.files
            .push(files.iter().map(|v| v.to_string()).collect());
        self
    }

    /// Builds TailcallExecutor with the provided runtime
    pub async fn build(self, runtime: TargetRuntime) -> Result<TailcallExecutor> {
        // Configuration reader initialization
        let reader = ConfigReader::init(runtime.clone());

        // Read configuration from files
        let mut config_module = reader.read_all(&self.files).await?;

        // Iterate over schema holders and merge configs
        for holder in self.schemas {
            let config = Config::from_source(holder.source, &holder.schema)?;
            let new_config_module = reader.resolve(config, holder.parent_dir.as_deref()).await?;
            config_module = config_module.merge_right(&new_config_module);
        }

        // Init AppContext
        let blueprint = Blueprint::try_from(&config_module)?;
        let app_ctx = AppContext::new(blueprint, runtime);
        let app_ctx = Arc::new(app_ctx);
        Ok(TailcallExecutor { config_module, app_ctx })
    }
}

impl TailcallExecutor {
    /// Validates configuration and performs optional schema validation
    pub async fn validate(
        &self,
        n_plus_one_queries: bool,
        schema: bool,
        ops: Vec<OperationQuery>, // TODO: check if we can carryout IO for OperationQuery in this function
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

    /// Executes GraphQL request
    pub async fn execute(self, req: Request<Body>) -> Result<Response<Body>> {
        if self.app_ctx.blueprint.server.enable_batch_requests {
            handle_request::<GraphQLBatchRequest>(req, self.app_ctx.clone()).await
        } else {
            handle_request::<GraphQLRequest>(req, self.app_ctx.clone()).await
        }
    }
}

impl Default for TailcallBuilder {
    fn default() -> Self {
        Self::new()
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
