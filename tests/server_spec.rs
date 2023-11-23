mod graphql_mock;

use async_graphql::Pos;
use serde::{Deserialize, Serialize};

use crate::graphql_mock::start_mock_server;

async fn initiate_test_server(mock_schema_path: String) {
  let config = tailcall::config::Config::from_file_or_url([mock_schema_path].iter())
    .await
    .unwrap();
  tailcall::http::start_server(config)
    .await
    .expect("Server failed to start");
}

#[derive(Serialize, Deserialize, Debug)]
struct Error {
  message: String,
  locations: Vec<Pos>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Resp {
  pub data: Option<Data>,
  pub errors: Option<Vec<Error>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
  pub post: Post,
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
  pub title: String,
  pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
  pub name: String,
  pub username: String,
}

#[tokio::test]
async fn test_server() {
  let _mock_server = start_mock_server();
  let backend_schema_path = "tests/graphql_mock/test-graphql-request-backend.graphql";
  let frontend_schema_path = "tests/graphql_mock/test-graphql-request-frontend.graphql";

  tokio::spawn(initiate_test_server(backend_schema_path.into()));
  tokio::spawn(initiate_test_server(frontend_schema_path.into()));

  tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

  let http_client = reqwest::Client::new();
  let success_query = "{\"query\":\"query { post(id: 11) { title user { name username } } }\"}";

  let api_request = http_client
    .post("http://localhost:8001/graphql")
    .header("Content-Type", "application/json")
    .body(success_query);

  let response = api_request.send().await.expect("Failed to send request");
  let json = response.json::<Resp>().await.unwrap();
  assert_eq!(json.data.unwrap().post.user.name, "Leanne Graham");

  let fail_404_query = "{\"query\":\"query { post(id: 1254) { title user { name username } } }\"}";

  let api_request = http_client
    .post("http://localhost:8001/graphql")
    .header("Content-Type", "application/json")
    .body(fail_404_query);

  let response = api_request.send().await.expect("Failed to send request");
  let json = response.json::<Resp>().await.unwrap();
  assert_eq!(
    json.errors.unwrap()[0].message,
    "IOException: Request error: HTTP status client error (404 Not Found) for url (http://localhost:3080/posts/1254)"
  );
}
