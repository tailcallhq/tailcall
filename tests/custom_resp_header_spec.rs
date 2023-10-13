// Integration tests for the API server.
mod integration_tests {

  // Helper function to start the test server.
  async fn initiate_test_server(mock_schema_path: String) -> &'static str {
    let config = tailcall::config::Config::from_file_paths([mock_schema_path].iter())
      .await
      .unwrap();
    tailcall::http::start_server(config)
      .await
      .expect("Server failed to start");
    "Success"
  }

  #[tokio::test]
  async fn verify_response_headers() {
    // Define the path to our mock GraphQL schema for testing.
    let schema_path = "tests/graphql_mock/test-custom-headers.graphql";

    // Start the background server with the provided schema.
    tokio::spawn(initiate_test_server(schema_path.into()));

    // Provide a small delay to ensure the server has started.
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

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
}
