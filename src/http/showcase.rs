use std::collections::HashMap;

use anyhow::Result;
use async_graphql::ServerError;
use hyper::{Body, Request, Response};
use serde::de::DeserializeOwned;
use url::Url;

use super::AppContext;
use crate::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};
use crate::blueprint::Blueprint;
use crate::config::reader::ConfigReader;
use crate::target_runtime::TargetRuntime;

pub async fn create_app_ctx<T: DeserializeOwned + GraphQLRequestLike>(
    req: &Request<Body>,
    runtime: TargetRuntime,
    enable_fs: bool,
) -> Result<Result<AppContext, Response<Body>>> {
    let config_url = req.uri()
        .query()
        .and_then(|x| serde_qs::from_str::<HashMap<String, String>>(x).ok())
        .and_then(|x| x.get("config").cloned());

    let config_url = if let Some(config_url) = config_url {
        config_url
    } else {
        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new("No Config URL specified", None);
        response.errors = vec![server_error];
        return Ok(Err(GraphQLResponse::from(response).to_response()?));
    };

    if !enable_fs && Url::parse(&config_url).is_err() {
        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new("Invalid Config URL specified", None);
        response.errors = vec![server_error];
        return Ok(Err(GraphQLResponse::from(response).to_response()?));
    }

    let reader = ConfigReader::init(runtime.clone());
    let config = match reader.read(config_url).await {
        Ok(config) => config,
        Err(e) => {
            let mut response = async_graphql::Response::default();
            let server_error = ServerError::new(format!("Failed to read config: {}", e), None);
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

    Ok(Ok(AppContext::new(blueprint, runtime)))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hyper::Request;
    use serde_json::json;

    use crate::async_graphql_hyper::GraphQLRequest;
    use crate::blueprint::Upstream;
    use crate::cli::init_runtime;
    use crate::http::handle_request;
    use crate::http::showcase::create_app_ctx;

    #[tokio::test]
    async fn works_with_file() {
        let req = Request::builder()
            .method("POST")
            .uri("http://upstream/showcase/graphql?config=.%2Ftests%2Fhttp%2Fconfig%2Fsimple.graphql")
            .body(hyper::Body::from(json!({
                "query": "query { user { name } }"
            }).to_string()))
            .unwrap();

        let runtime = init_runtime(&Upstream::default(), None);
        let app = create_app_ctx::<GraphQLRequest>(&req, runtime, true)
            .await
            .unwrap()
            .unwrap();

        let req = Request::builder()
            .method("POST")
            .uri("http://upstream/graphql?config=.%2Ftests%2Fhttp%2Fconfig%2Fsimple.graphql")
            .body(hyper::Body::from(
                json!({
                    "query": "query { user { name } }"
                })
                .to_string(),
            ))
            .unwrap();

        let res = handle_request::<GraphQLRequest>(req, Arc::new(app))
            .await
            .unwrap();

        println!("{:#?}", res);
        assert!(res.status().is_success())
    }
}
