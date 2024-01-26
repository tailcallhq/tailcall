use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_graphql::ServerError;
use hyper::{Body, Request, Response};
use serde::de::DeserializeOwned;
use url::Url;

use super::AppContext;
use crate::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};
use crate::blueprint::Blueprint;
use crate::config::reader::ConfigReader;
use crate::{EntityCache, EnvIO, FileIO, HttpIO};

pub struct DummyFileIO;

#[async_trait::async_trait]
impl FileIO for DummyFileIO {
  async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> anyhow::Result<()> {
    Err(anyhow!("DummyFileIO"))
  }

  async fn read<'a>(&'a self, _path: &'a str) -> anyhow::Result<String> {
    Err(anyhow!("DummyFileIO"))
  }
}

pub struct DummyEnvIO;

impl EnvIO for DummyEnvIO {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
}

pub async fn showcase_get_app_ctx<T: DeserializeOwned + GraphQLRequestLike>(
  req: &Request<Body>,
  http: Arc<dyn HttpIO + Send + Sync>,
  env: Option<Arc<dyn EnvIO>>,
  file: Option<Arc<dyn FileIO + Send + Sync>>,
  cache: Arc<EntityCache>,
) -> Result<Result<AppContext, Response<Body>>> {
  let url = Url::parse(&req.uri().to_string())?;
  let mut query = url.query_pairs();

    let config_url = if let Some(pair) = query.find(|x| x.0 == "config") {
        pair.1.to_string()
    } else {
        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new("No Config URL specified", None);
        response.errors = vec![server_error];
        return Ok(Err(GraphQLResponse::from(response).to_response()?));
    };

    let config = if let Some(file) = file {
        let reader = ConfigReader::init(file, http.clone());
        reader.read(config_url).await
    } else {
        let reader = ConfigReader::init(Arc::new(DummyFileIO), http.clone());
        reader.read(config_url).await
    };

    let config = match config {
        Ok(config) => config,
        Err(e) => {
            let mut response = async_graphql::Response::default();
            let server_error = if e.to_string() == "DummyFileIO" {
                ServerError::new("Invalid Config URL specified", None)
            } else {
                ServerError::new(format!("{}", e), None)
            };
            response.errors = vec![server_error];
            return Ok(Err(GraphQLResponse::from(response).to_response()?));
        }
    };

    let blueprint = match Blueprint::try_from(&config) {
        Ok(blueprint) => blueprint,
        Err(e) => {
            let mut response = async_graphql::Response::default();
            let server_error = ServerError::new(format!("{}", e), None);
            response.errors = vec![server_error];
            return Ok(Err(GraphQLResponse::from(response).to_response()?));
        }
    };

    let env = env.unwrap_or_else(|| Arc::new(DummyEnvIO));

    Ok(Ok(AppContext::new(
        blueprint,
        http.clone(),
        http,
        env,
        cache,
        None,
    )))
}
