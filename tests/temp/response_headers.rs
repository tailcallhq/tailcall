use crate::http::start_server;

async fn test_custom_header_response(){
    let file_path = "graphql/passed/test-custom-response-headers.graphql";
    start_server(&file_path).await?;

    let query = "{data {post1 { title}}}";

    let res = reqwest::Client::new()
        .post("http://localhost:3000/graphql")
        .json(&json!({ "query": query }));

    let res = res.send().await.unwrap();
    let headers = res.headers();

    assert_eq!(headers.get("x-custom-header").unwrap(), "custom-value");


}





