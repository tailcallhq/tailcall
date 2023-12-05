use futures::future;
use tailcall::config::Config;
use tailcall::http::{start_server, ServerControl, ServerMessage};

#[tokio::test]
async fn server_start() {
  use reqwest;
  use reqwest::Client;
  use serde_json::json;

  let (server_control, server_up_sender, shutdown_sender) = ServerControl::new();

  let file_paths = vec!["tests/server/config/server-start.graphql".to_string()];
  let config = Config::from_file_or_url(file_paths.iter()).await.unwrap();
  tokio::spawn(async {
    start_server(config, server_up_sender, server_control.shutdown.receiver)
      .await
      .unwrap();
  });

  match server_control.server_up.receiver.await {
    Ok(ServerMessage::ServerUp) => {
      println!("Server started");
    }
    _ => panic!("Server did not start up correctly"),
  }

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
      response_body
    });
    tasks.push(task);
  }

  let tasks = future::join_all(tasks).await;

  for task in tasks {
    let response_body = match task {
      Ok(body) => body,
      Err(_) => panic!("Task panicked"),
    };
    let expected_response = json!({
        "data": {
            "greet": "Hello World!"
        }
    });
    assert_eq!(response_body, expected_response, "Unexpected response from server");
  }

  shutdown_sender.send(ServerMessage::Shutdown).ok();
}

#[tokio::test]
async fn server_start_http2() {
  use reqwest;
  use reqwest::Client;
  use serde_json::json;

  let (server_control, server_up_sender, shutdown_sender) = ServerControl::new();

  tokio::spawn(async {
    let file_paths = vec!["tests/server/config/server-start-http2.graphql".to_string()];
    let config = Config::from_file_or_url(file_paths.iter()).await.unwrap();
    start_server(config, server_up_sender, server_control.shutdown.receiver)
      .await
      .unwrap();
  });

  match server_control.server_up.receiver.await {
    Ok(ServerMessage::ServerUp) => (),
    _ => panic!("Server did not start up correctly"),
  }

  // required since our cert is self signed
  let client = Client::builder().danger_accept_invalid_certs(true).build().unwrap();
  let query = json!({
      "query": "{ greet }"
  });

  let mut tasks = vec![];
  for _ in 0..100 {
    let client = client.clone();
    let query = query.clone();
    let task = tokio::spawn(async move {
      let send_request = || async { client.post("https://0.0.0.0:8000/graphql").json(&query).send().await };
      let response = send_request().await.expect("Failed to send request");
      let response_body: serde_json::Value = response.json().await.expect("Failed to parse response body");
      response_body
    });
    tasks.push(task);
  }

  let tasks = future::join_all(tasks).await;

  for task in tasks {
    let response_body = match task {
      Ok(body) => body,
      Err(_) => panic!("Task panicked"),
    };
    let expected_response = json!({
        "data": {
            "greet": "Hello World!"
        }
    });
    assert_eq!(response_body, expected_response, "Unexpected response from server");
  }

  shutdown_sender.send(ServerMessage::Shutdown).ok();
}
