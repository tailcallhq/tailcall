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

struct DummyFileIO;

#[async_trait::async_trait]
impl FileIO for DummyFileIO {
    async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> anyhow::Result<()> {
        Err(anyhow!("DummyFileIO"))
    }

    async fn read<'a>(&'a self, _path: &'a str) -> anyhow::Result<String> {
        Err(anyhow!("DummyFileIO"))
    }
}

struct DummyEnvIO;

impl EnvIO for DummyEnvIO {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
}

pub struct ShowcaseResources {
    pub http: Arc<dyn HttpIO + Send + Sync>,
    pub env: Option<Arc<dyn EnvIO>>,
    pub file: Option<Arc<dyn FileIO + Send + Sync>>,
    pub cache: Arc<EntityCache>,
}

pub async fn showcase_get_app_ctx<T: DeserializeOwned + GraphQLRequestLike>(
    req: &Request<Body>,
    resources: ShowcaseResources,
) -> Result<Result<AppContext, Response<Body>>> {
    let url = Url::parse(&req.uri().to_string())?;
    let mut query = url.query_pairs();

    let http = resources.http;

    let config_url = if let Some(pair) = query.find(|x| x.0 == "config") {
        pair.1.to_string()
    } else {
        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new("No Config URL specified", None);
        response.errors = vec![server_error];
        return Ok(Err(GraphQLResponse::from(response).to_response()?));
    };

    let config = if let Some(file) = resources.file {
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
            let server_error = if format!("{:?}", e.source()) == "Some(\"DummyFileIO\")" {
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

    let env = resources.env.unwrap_or_else(|| Arc::new(DummyEnvIO));

    Ok(Ok(AppContext::new(
        blueprint,
        http.clone(),
        http,
        env,
        resources.cache,
    )))
}

#[cfg(test)]
mod tests {
    use hyper::Request;
    use serde_json::json;
    use std::sync::Arc;

    use crate::{async_graphql_hyper::GraphQLRequest, cli::{init_env, init_file, init_http, init_in_memory_cache}, config::Upstream, http::{handle_request, showcase::{DummyEnvIO, DummyFileIO}, showcase_get_app_ctx, ShowcaseResources}, EnvIO as _, FileIO as _};

    #[test]
    fn dummy_env_works() {
        let env = DummyEnvIO;

        assert_eq!(env.get("PATH").is_none(), true);
    }

    #[tokio::test]
    async fn dummy_file_works() {
        let file = DummyFileIO;

        assert_eq!(file.read("./README.md").await.is_err(), true);
        assert_eq!(file.write("./README.md", b"hello world").await.is_err(), true);
    }

    #[tokio::test]
    async fn works_with_file() {
        let req = Request::builder()
            .method("POST")
            .uri("http://upstream/showcase/graphql?config=.%2Ftests%2Fhttp%2Fconfig%2Fsimple.graphql")
            .body(hyper::Body::from(json!({
                "query": "query { user { name } }"
            }).to_string()))
            .unwrap();

        let app = showcase_get_app_ctx::<GraphQLRequest>(
            &req,
            ShowcaseResources {
                http: init_http(&Upstream::default(), None),
                env: Some(init_env()),
                file: Some(init_file()),
                cache: Arc::new(init_in_memory_cache()),
            },
        ).await.unwrap().unwrap();

        let res = handle_request::<GraphQLRequest>(req, Arc::new(app)).await.unwrap();

        println!("{:#?}", res);
    }
}
