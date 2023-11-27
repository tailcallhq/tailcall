use tailcall::config::Config;
use tailcall::http::start_server;

#[tokio::test]
async fn server_start() {
  use reqwest;
  use reqwest::Client;
  use serde_json::json;
  use tokio::sync::oneshot;
  let (tx, rx) = oneshot::channel::<bool>();

  tokio::spawn(async {
    let file_paths = vec!["tests/server/config/server-start.graphql".to_string()];
    let config = Config::from_file_or_url(file_paths.iter()).await.unwrap();
    start_server(config, tx).await.unwrap();
  });

  rx.await.unwrap();

  let client = Client::new();
  let query = json!({
      "query": "{ greet }"
  });

  let mut tasks = vec![];
  for _ in 0..100 {
    let client = client.clone();
    let query = query.clone();
    let task = tokio::spawn(async move {
      let send_request = || async { client.post("http://localhost:8000/graphql").json(&query).send().await };
      let response = send_request().await.expect("Failed to send request");
      let response_body: serde_json::Value = response.json().await.expect("Failed to parse response body");
      assert_eq!(response_body["data"]["greet"], "Hello World!");
    });
    tasks.push(task);
  }
}
