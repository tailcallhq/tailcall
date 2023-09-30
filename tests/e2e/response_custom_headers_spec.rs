#[cfg(test)]
mod test {
  use serde_json::json;

  async fn background_server(file_path: String) -> &'static str {
    tailcall::http::start_server(&file_path).await.unwrap();

    "Ok"
  }

  #[tokio::test]
  async fn test_custom_header_response() {
    let file_path = "tests/e2e/graphql_mock_schemas/test-custom-response-headers.graphql";
    tokio::spawn(background_server(file_path.to_string()));
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let query = "{data {post1 { title}}}";
    let res = reqwest::Client::new()
      .post("http://localhost:8000/graphql")
      .json(&json!({ "query": query }));

    let res = res.send().await.unwrap();
    let headers = res.headers();
    assert_eq!(headers.get("x_custom_header").unwrap(), "custom-value");
  }
}
