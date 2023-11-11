// Integration tests for the API server.
mod integration_tests {
  use tailcall::http::{start_server, start_server_with_polling};

  // Helper function to start the test server.
  async fn initiate_test_server(mock_schema_path: String, poll_interval: Option<u64>) -> &'static str {
    let config = tailcall::config::Config::from_file_or_url([mock_schema_path.clone()].iter())
      .await
      .unwrap();
    match poll_interval {
      Some(poll_interval) => {
        match start_server_with_polling(&config, [mock_schema_path].to_vec(), poll_interval).await {
          Ok(_) => {}
          Err(_) => {
            start_server(config).await.unwrap();
          }
        }
      }
      _ => {
        start_server(config).await.unwrap();
      }
    }
    "Success"
  }

  async fn verify_response_headers(schema_path: &str, poll_interval: Option<u64>) {
    // Start the background server with the provided schema.
    tokio::spawn(initiate_test_server(schema_path.into(), poll_interval));

    // Provide a small delay to ensure the server has started.
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await; // need to provide a bit of delay depending upon internet speed and response time

    let http_client = reqwest::Client::new();
    let query_data = "{\"query\":\"query { item { id name } }\"}";

    let api_request = http_client
      .post("http://localhost:8000/graphql")
      .header("Content-Type", "application/json")
      .body(query_data);

    let response = api_request.send().await.expect("Failed to send request");

    let response_headers = response.headers();
    assert_eq!(response_headers.get("x-id").unwrap(), "1");
    assert_eq!(response_headers.get("x-value").unwrap(), "value");
  }

  #[tokio::test]
  async fn test_verify_response_headers() {
    let schema_path = "tests/graphql_mock/test-custom-headers.graphql";
    verify_response_headers(schema_path, None).await;
  }

  #[tokio::test]
  async fn test_verify_response_headers_over_http() {
    let schema_path =
      "https://raw.githubusercontent.com/tailcallhq/tailcall/main/tests/graphql_mock/test-custom-headers.graphql";
    let poll_interval = 10;
    verify_response_headers(schema_path, Some(poll_interval)).await;
  }
}
