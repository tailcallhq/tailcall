use std::str::FromStr;
use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::async_graphql_hyper::GraphQLRequestLike;
use crate::http::parser::Parser;
use crate::http::request_handler::{create_request_context, update_cache_control_header, update_response_headers};
use crate::http::ServerContext;

pub async fn api_request<T: DeserializeOwned + GraphQLRequestLike>(
  req: Request<Body>,
  server_ctx: &ServerContext,
) -> anyhow::Result<Response<Body>> {
  if let Some(query) = req.uri().path_and_query() {
    let query = query.as_str();
    let mut parser = Parser::from_path(query)?;
    let request = parser.parse::<T>(&server_ctx.blueprint.definitions);
    match request {
      Ok(request) => {
        let req_ctx = Arc::new(create_request_context(&req, server_ctx));
        let response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;
        let mut response = update_cache_control_header(response, server_ctx, req_ctx).to_response()?;
        update_response_headers(&mut response, server_ctx);
        Ok(response)
      }
      Err(err) => {
        log::error!("Failed to parse request: {query}");
        let status_code = if err.to_string().starts_with("404") {
          StatusCode::NOT_FOUND
        } else {
          StatusCode::BAD_REQUEST
        };
        let resp = Response::builder()
          .status(status_code)
          .body(Body::from(make_error_json(format!("Unexpected API Request: {}", err))));
        Ok(resp?)
      }
    }
  } else {
    log::error!("Failed to parse request, invalid url",);
    let response = Response::builder()
      .status(StatusCode::BAD_REQUEST)
      .body(Body::from(make_error_json(
        "Failed to parse request, invalid url".to_string(),
      )));
    Ok(response?)
  }
}

fn make_error_json(err: String) -> String {
  let mut mp = Map::new();
  mp.insert(
    "errors".to_string(),
    Value::from_str(&format!("[{{\"message\": \"{err}\"}}]")).unwrap(),
  );
  mp.insert("data".to_string(), Value::Null);
  Value::Object(mp).to_string()
}
