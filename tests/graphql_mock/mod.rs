use std::fs;

use regex::Regex;
use serde_json::Value;

pub fn start_mock_server() -> mockito::Server {
  mockito::Server::new_with_port(3080)
}

pub fn setup_mocks(mock_server: &mut mockito::Server) {
  let users_json = fs::read_to_string("tests/data/users.json").unwrap();
  let users: Vec<serde_json::Value> = serde_json::from_str(users_json.as_str()).unwrap();
  let users_2: Vec<serde_json::Value> = serde_json::from_str(users_json.as_str()).unwrap();
  let users_posts_json = fs::read_to_string("tests/data/user-posts.json").unwrap();
  let user_posts: serde_json::Value = serde_json::from_str(users_posts_json.as_str()).unwrap();
  let user_posts_2: serde_json::Value = serde_json::from_str(users_posts_json.as_str()).unwrap();
  let posts_json = fs::read_to_string("tests/data/posts.json").unwrap();
  let introspection_result = fs::read_to_string("tests/data/introspection-result.json").unwrap();

  mock_server
    .mock("GET", "/users")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(users_json)
    .create();

  mock_server
    .mock("GET", mockito::Matcher::Regex(r"^/users/(\d+)$".to_string()))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body_from_request(move |req| {
      let id = get_id_from_path(req.path(), Regex::new(r"^/users/(\d+)$").unwrap());
      let user = users.iter().find(|x| x["id"].as_u64().unwrap() == id).unwrap();
      user.to_string().into()
    })
    .create();

  mock_server
    .mock("GET", "/posts?")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(posts_json)
    .create();

  mock_server
    .mock("GET", mockito::Matcher::Regex(r"^/users/(\d+)/posts$".to_string()))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body_from_request(move |req| {
      let id = get_id_from_path(req.path(), Regex::new(r"^/users/(\d+)/posts$").unwrap());
      if let Value::Object(obj) = &user_posts {
        let posts = obj.get(&id.to_string()).unwrap();
        posts.to_string().into()
      } else {
        Value::Null.to_string().into()
      }
    })
    .create();

  mock_server
    .mock("GET", mockito::Matcher::Regex(r"^/firstPost".to_string()))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body_from_request(move |_| {
      if let Value::Object(obj) = &user_posts_2 {
        let mut first_posts: Vec<Value> = Vec::new();
        for u in &users_2 {
          let id = u["id"].as_u64().unwrap();
          let posts = obj.get(&id.to_string()).unwrap();
          match posts {
            Value::Array(posts) => {
              if !posts.is_empty() {
                first_posts.push(posts[0].clone());
              }
            }
            _ => return Value::Null.to_string().into(),
          }
        }
        Value::Array(first_posts).to_string().into()
      } else {
        Value::Null.to_string().into()
      }
    })
    .create();

  mock_server
    .mock("GET", "/defaultPropertyResolver")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"a": 1}"#)
    .create();

  mock_server
    .mock("POST", "/graphql")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(introspection_result)
    .create();
}

pub fn get_id_from_path(path: &str, re: Regex) -> u64 {
  let caps = re.captures(path).unwrap();
  let id = &caps[1];
  id.parse::<u64>().unwrap()
}
