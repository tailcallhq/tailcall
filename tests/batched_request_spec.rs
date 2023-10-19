mod integration_tests {
  use serde::{Deserialize, Serialize};
  use serde_json::json;

  async fn make_request(http_client: reqwest::Client, body: String) -> reqwest::Response {
    http_client
      .post("http://localhost:8000/graphql")
      .header("Content-Type", "application/json")
      .body(body)
      .send()
      .await
      .expect("Failed to send request")
  }

  async fn initiate_test_server(mock_schema_path: String) -> &'static str {
    let config = tailcall::config::Config::from_file_paths([mock_schema_path].iter())
      .await
      .unwrap();
    tailcall::http::start_server(config)
      .await
      .expect("Server failed to start");
    "Success"
  }

  #[derive(Serialize, Deserialize, Debug)]
  struct Resp {
    pub data: Data,
  }

  #[derive(Serialize, Deserialize, Debug)]
  struct Data {
    pub post: Post,
  }

  #[derive(Serialize, Deserialize, Debug)]
  struct Post {
    pub title: String,
  }

  #[tokio::test]
  async fn test_requests_with_batch_requests_enabled() {
    let schema_path = "tests/graphql_mock/test-batched-request.graphql";

    tokio::spawn(initiate_test_server(schema_path.into()));
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    let http_client = reqwest::Client::new();

    let query1 = json!({"query": "query { post(id: 1) { title } }"});
    let query2 = json!({"query": "query { post(id: 2) { title } }"});

    let batched_query = format!("[{},{}]", query1, query2);
    let response = make_request(http_client.clone(), batched_query).await;
    let json = response.json::<Vec<Resp>>().await.unwrap();
    assert_eq!(json.len(), 2);
    assert_eq!(
      json.first().unwrap().data.post.title,
      "sunt aut facere repellat provident occaecati excepturi optio reprehenderit"
    );
    assert_eq!(json.get(1).unwrap().data.post.title, "qui est esse");

    let unbatched_query = format!("{}", query1);
    let response = make_request(http_client, unbatched_query).await;
    let json = response.json::<Resp>().await.unwrap();
    assert_eq!(
      json.data.post.title,
      "sunt aut facere repellat provident occaecati excepturi optio reprehenderit"
    );
  }
}
