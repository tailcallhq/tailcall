use std::collections::HashMap;

use anyhow::Result;
use async_graphql::ServerError;
use hyper::{Response};
use serde::de::DeserializeOwned;
use url::Url;

use super::{AppContext, Request};
use crate::core::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};
use crate::core::blueprint::Blueprint;
use crate::core::Body;
use crate::core::config::reader::ConfigReader;
use crate::core::rest::EndpointSet;
use crate::core::runtime::TargetRuntime;

pub async fn create_app_ctx<T: DeserializeOwned + GraphQLRequestLike>(
    req: &Request,
    runtime: TargetRuntime,
    enable_fs: bool,
) -> Result<Result<AppContext, Response<Body>>> {
    let config_url = req
        .uri
        .query()
        .and_then(|x| serde_qs::from_str::<HashMap<String, String>>(x).ok())
        .and_then(|x| x.get("config").cloned());

    let config_url = if let Some(config_url) = config_url {
        config_url
    } else {
        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new("No Config URL specified", None);
        response.errors = vec![server_error];
        return Ok(Err(GraphQLResponse::from(response).into_response()?));
    };

    if !enable_fs && Url::parse(&config_url).is_err() {
        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new("Invalid Config URL specified", None);
        response.errors = vec![server_error];
        return Ok(Err(GraphQLResponse::from(response).into_response()?));
    }

    let reader = ConfigReader::init(runtime.clone());
    let config = match reader.read(config_url).await {
        Ok(config) => config,
        Err(e) => {
            let mut response = async_graphql::Response::default();
            let server_error = ServerError::new(format!("Failed to read config: {}", e), None);
            response.errors = vec![server_error];
            return Ok(Err(GraphQLResponse::from(response).into_response()?));
        }
    };

    let blueprint = match Blueprint::try_from(&config) {
        Ok(blueprint) => blueprint,
        Err(e) => {
            let mut response = async_graphql::Response::default();
            let server_error = ServerError::new(format!("{}", e), None);
            response.errors = vec![server_error];
            return Ok(Err(GraphQLResponse::from(response).into_response()?));
        }
    };

    Ok(Ok(AppContext::new(
        blueprint,
        runtime,
        EndpointSet::default(),
    )))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use bytes::Bytes;
    use hyper::Method;

    use serde_json::json;

    use crate::core::async_graphql_hyper::GraphQLRequest;
    use crate::core::Body;
    use crate::core::http::{handle_request, Request};
    use crate::core::http::showcase::create_app_ctx;

    #[tokio::test]
    async fn works_with_file() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("http://upstream/showcase/graphql?config=.%2Ftests%2Fhttp%2Fconfig%2Fsimple.graphql".parse().unwrap())
            .body(Bytes::from(json!({
                "query": "query { user { name } }"
            }).to_string()))
            .unwrap();

        let runtime = crate::core::runtime::test::init(None);
        let app = create_app_ctx::<GraphQLRequest>(&req, runtime, true)
            .await
            .unwrap()
            .unwrap();

        let req = Request::builder()
            .method(Method::POST)
            .uri("http://upstream/graphql?config=.%2Ftests%2Fhttp%2Fconfig%2Fsimple.graphql".parse().unwrap())
            .body(Bytes::from(
                json!({
                    "query": "query { user { name } }"
                })
                .to_string(),
            ))
            .unwrap();

        let res = handle_request::<GraphQLRequest>(req, Arc::new(app))
            .await
            .unwrap();

        assert!(res.status().is_success())
    }
}
