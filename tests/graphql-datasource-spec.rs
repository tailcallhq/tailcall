use serde::{Deserialize, Serialize};

async fn initiate_test_server(mock_schema_path: String) -> &'static str {
  let config = tailcall::config::Config::from_file_or_url([mock_schema_path].iter())
    .await
    .unwrap();
  tailcall::http::start_server(config)
    .await
    .expect("Server failed to start");
  "Success"
}

#[derive(Serialize, Deserialize, Debug)]
struct Resp {
  pub data: Data,
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
async fn test_graphql_datasource() {
  let upstream_schema_path = "tests/graphql_mock/graphql-datasource/upstream.graphql";
  let composed_schema_path = "tests/graphql_mock/graphql-datasource/composed.graphql";

  tokio::spawn(initiate_test_server(upstream_schema_path.into()));
  tokio::spawn(initiate_test_server(composed_schema_path.into()));

  tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

  let http_client = reqwest::Client::new();
  let query_data = "{\"query\":\"query { post(id: 1) { title user { name username } } }\"}";

  let api_request = http_client
    .post("http://localhost:8001/graphql")
    .header("Content-Type", "application/json")
    .body(query_data);

  let response = api_request.send().await.expect("Failed to send request");
  let json = response.json::<Resp>().await.unwrap();
  assert_eq!(json.data.post.user.name, "Leanne Graham");
}
