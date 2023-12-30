use reqwest::Client;
use serde_json::json;
use tc_cli::http::server::Server;
use tc_core::config::Config;

async fn test_server(configs: &[&str], url: &str) {
  let config = Config::read_from_files(configs.iter()).await.unwrap();
  let mut server = Server::new(config);
  let server_up_receiver = server.server_up_receiver();

  tokio::spawn(async move {
    server.start().await.unwrap();
  });

  server_up_receiver.await.expect("Server did not start up correctly");

  // required since our cert is self signed
  let client = Client::builder().danger_accept_invalid_certs(true).build().unwrap();
  let query = json!({
      "query": "{ greet }"
  });

  let mut tasks = vec![];
  for _ in 0..100 {
    let client = client.clone();
    let url = url.to_owned();
    let query = query.clone();

    let task: tokio::task::JoinHandle<Result<serde_json::Value, anyhow::Error>> = tokio::spawn(async move {
      let response = client.post(url).json(&query).send().await?;
      let response_body: serde_json::Value = response.json().await?;
      Ok(response_body)
    });
    tasks.push(task);
  }

  for task in tasks {
    let response_body = task
      .await
      .expect("Spawned task should success")
      .expect("Request should success");
    let expected_response = json!({
        "data": {
            "greet": "Hello World!"
        }
    });
    assert_eq!(response_body, expected_response, "Unexpected response from server");
  }
}

#[tokio::test]
async fn server_start() {
  test_server(
    &["tests/server/config/server-start.graphql"],
    "http://localhost:8800/graphql",
  )
  .await
}

#[tokio::test]
async fn server_start_http2() {
  test_server(
    &["tests/server/config/server-start-http2.graphql"],
    "https://localhost:8801/graphql",
  )
  .await
}
