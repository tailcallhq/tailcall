mod graphql_mock;

use async_graphql::Response;
use serde::{Deserialize, Serialize};

use crate::graphql_mock::start_mock_server;

async fn initiate_test_server(mock_schema_path: &str) {
  let config = tailcall::config::Config::from_file_or_url([mock_schema_path].iter())
    .await
    .unwrap();
  tailcall::http::start_server(config)
    .await
    .expect("Server failed to start");
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

  tokio::spawn(initiate_test_server(
    "tests/graphql_mock/test-graphql-request-backend.graphql",
  ));
  tokio::spawn(initiate_test_server(
    "tests/graphql_mock/test-graphql-request-frontend.graphql",
  ));

  tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

  let http_client = reqwest::Client::new();
  let query = "{\"query\":\"query { post(id: 11) { title user { name username } } }\"}";

  let req = http_client
    .post("http://localhost:8001/graphql")
    .header("Content-Type", "application/json")
    .body(query);

  let res = req.send().await.expect("Failed to send request");
  let json = res.json::<Response>().await.unwrap();
  let data: Data = serde_json::from_value(json.data.into_json().unwrap()).unwrap();
  assert_eq!(data.post.user.name, "Leanne Graham");

  let query = "{\"query\":\"query { post(id: 1254) { title user { name username } } }\"}";

  let req = http_client
    .post("http://localhost:8001/graphql")
    .header("Content-Type", "application/json")
    .body(query);

  let res = req.send().await.expect("Failed to send request");
  let json = res.json::<Response>().await.unwrap();
  assert_eq!(
    json.errors[0].message,
    "IOException: Request error: HTTP status client error (404 Not Found) for url (http://localhost:3080/posts/1254)"
  );

  let query = "{\"query\":\"query { nonexisting }\"}";

  let req = http_client
    .post("http://localhost:8001/graphql")
    .header("Content-Type", "application/json")
    .body(query);

  let res = req.send().await.expect("Failed to send request");
  let json = res.json::<Response>().await.unwrap();

  // TODO: for some reason async_graphql returns nothing for wrong queries instead of showing error
  assert_eq!(json.data.into_json().unwrap().to_string(), "{}");
}
