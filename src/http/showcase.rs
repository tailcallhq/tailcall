use std::collections::HashMap;

use anyhow::Result;
use async_graphql::ServerError;
use hyper::{Body, Request, Response};
use url::Url;

use crate::async_graphql_hyper::GraphQLResponse;
use crate::builder::{TailcallBuilder, TailcallExecutor};
use crate::runtime::TargetRuntime;
use crate::valid::ValidationError;

pub async fn create_tailcall_executor(
    req: &Request<Body>,
    runtime: TargetRuntime,
    enable_fs: bool,
) -> Result<Result<TailcallExecutor, Response<Body>>, ValidationError<String>> {
    let config_url = req
        .uri()
        .query()
        .and_then(|x| serde_qs::from_str::<HashMap<String, String>>(x).ok())
        .and_then(|x| x.get("config").cloned());

    let config_url = if let Some(config_url) = config_url {
        config_url
    } else {
        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new("No Config URL specified", None);
        response.errors = vec![server_error];
        return Ok(Err(GraphQLResponse::from(response).into_response().map_err(|e|ValidationError::new(e.to_string()))?));
    };

    if !enable_fs && Url::parse(&config_url).is_err() {
        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new("Invalid Config URL specified", None);
        response.errors = vec![server_error];
        return Ok(Err(GraphQLResponse::from(response).into_response().map_err(|e|ValidationError::new(e.to_string()))?));
    }

    let tc_builder = TailcallBuilder::init(runtime.clone());
    let tailcall = match tc_builder.with_config_paths(&[config_url]).await {
        Ok(tailcall) => tailcall,
        Err(e) => {
            let mut response = async_graphql::Response::default();
            let server_error = ServerError::new(format!("Failed to read config: {}", e), None);
            response.errors = vec![server_error];
            return Ok(Err(GraphQLResponse::from(response).into_response().map_err(|e|ValidationError::new(e.to_string()))?));
        }
    };

    Ok(Ok(tailcall))
}

#[cfg(test)]
mod tests {
    use hyper::Request;
    use serde_json::json;

    use crate::http::showcase::create_tailcall_executor;

    #[tokio::test]
    async fn works_with_file() {
        let req = Request::builder()
            .method("POST")
            .uri("http://upstream/showcase/graphql?config=.%2Ftests%2Fhttp%2Fconfig%2Fsimple.graphql")
            .body(hyper::Body::from(json!({
                "query": "query { user { name } }"
            }).to_string()))
            .unwrap();

        let runtime = crate::runtime::test::init(None);
        let tailcall = create_tailcall_executor(&req, runtime, true)
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

        let res = tailcall.execute(req).await.unwrap();

        println!("{:#?}", res);
        assert!(res.status().is_success())
    }
}
