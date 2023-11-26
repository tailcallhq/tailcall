use tailcall::config::Config;
use tailcall::http::start_server;

#[tokio::test]
async fn server_start() {
  use reqwest;
  use reqwest::Client;
  use serde_json::json;

  tokio::spawn(async {
    let file_paths = vec!["tests/server/config/server-start.graphql".to_string()];
    let config = Config::from_file_or_url(file_paths.iter()).await.unwrap();
    start_server(config).await.unwrap();
  });

  let client = Client::new();
  let query = json!({
      "query": "{ greet }"
  });

  let send_request = || async {
    loop {
      let response = client.post("http://localhost:8000/graphql").json(&query).send().await;
      if let Err(err) = &response {
        if err.is_request() && format!("{}", err).contains("Connection refused") {
          continue;
        }
      }
      break response;
    }
  };

  for _ in 0..100 {
    let response = send_request().await.expect("Failed to send request");
    let response_body: serde_json::Value = response.json().await.expect("Failed to parse response body");
    assert_eq!(response_body["data"]["greet"], "Hello World!");
  }
}
