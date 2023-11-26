use tailcall::config::Config;
use tailcall::http::start_server;

#[tokio::test]
async fn server_start() {
  use reqwest::Client;
  use serde_json::json;

  tokio::spawn(async {
    let file_paths = vec!["tests/server/config/server-start.graphql".to_string()];
    let config = Config::from_file_or_url(file_paths.iter()).await.unwrap();
    start_server(config).await.unwrap();
  });

  // Give the server some time to start
  tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

  let client = Client::new();
  let query = json!({
      "query": "{ greet }"
  });

  for _ in 0..100 {
    let response = client
      .post("http://localhost:8000/graphql")
      .json(&query)
      .send()
      .await
      .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let response_body: serde_json::Value = response.json().await.expect("Failed to parse response body");
    assert_eq!(response_body["data"]["greet"], "Hello World!");
  }
}
