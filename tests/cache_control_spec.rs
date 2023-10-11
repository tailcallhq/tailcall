#[cfg(test)]
use std::fs;
use std::sync::Arc;

use async_graphql::Request;
use hyper::HeaderMap;
use pretty_assertions::assert_eq;
use std::time::Duration;
use tailcall::blueprint::Blueprint;
use tailcall::config::{Batch, Config};
use tailcall::http::{HttpDataLoader, RequestContext};

mod cache_control;

#[tokio::test]
async fn test_cache_control() -> std::io::Result<()> {
  let mut mock_server = cache_control::start_mock_server();
  cache_control::setup_mocks(&mut mock_server);

  let server_sdl = fs::read_to_string("tests/cache_control/graphql/server-sdl.graphql").unwrap();
  let query = fs::read_to_string("tests/cache_control/graphql/query.graphql").unwrap();

  let config = Config::from_sdl(server_sdl.as_str()).unwrap();
  let blueprint = Blueprint::try_from(&config).unwrap();
  let schema = blueprint.to_schema(&config.server);
  let headers = HeaderMap::new();
  let data_loader = Arc::new(HttpDataLoader::default().to_data_loader(Batch::default()));
  let req_ctx = Arc::new(
    RequestContext::default()
      .req_headers(headers)
      .server(config.server.clone())
      .data_loader(data_loader),
  );
  let req = Request::from(query).data(req_ctx.clone());
  schema.execute(req).await;

  assert_eq!(
    vec![Some(Duration::from_secs(300)), Some(Duration::from_secs(600))],
    req_ctx.get_max_ages()
  );

  Ok(())
}
