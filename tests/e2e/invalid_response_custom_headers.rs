#[cfg(test)]
mod test {
  use anyhow::Error;

  async fn background_server(file_path: String) -> Option<Error> {
    tailcall::http::start_server(&file_path).await.err()
  }

  #[tokio::test]
  async fn test_custom_header_response() {
    let file_path = "tests/e2e/graphql_mock_schemas/test-invalid-response-headers.graphql";
    let res = background_server(file_path.to_string()).await;
    assert_eq!(
      res.unwrap().to_string(),
      "Error: Invalid Configuration\nCaused by:\n  • invalid HTTP header name [at x-custom_header 1]"
    );
  }
}
