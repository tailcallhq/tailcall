#[cfg(test)]
mod test {

  async fn background_server(file_path: String) -> &'static str {
    tailcall::http::start_server(&file_path).await.unwrap();

    "Ok"
  }

  #[tokio::test]
  async fn test_custom_header_response() {
    let file_path = "tests/e2e/graphql_mock_schemas/test-custom-response-headers.graphql";
    tokio::spawn(background_server(file_path.to_string()));
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let client = reqwest::Client::new();

    let data = "{\"query\":\"query {\\n  post{\\n    id\\n    title\\n  }\\n}\"}";

    let request = client
      .post("http://localhost:8000/graphql")
      .header("Content-Type", "application/json")
      .body(data);

    let res = request.send().await.unwrap();

    let headers = res.headers();
    assert_eq!(headers.get("x-custom_header_1").unwrap(), "custom-value_1");
    assert_eq!(headers.get("x-custom_header_3").unwrap(), "custom-value 3");
  }
}
