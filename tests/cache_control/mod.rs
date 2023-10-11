use std::fs;

pub fn start_mock_server() -> mockito::Server {
  mockito::Server::new_with_port(8000)
}

pub fn setup_mocks(mock_server: &mut mockito::Server) {
  let posts_json = fs::read_to_string("tests/cache_control/data/posts.json").unwrap();
  let user_json = fs::read_to_string("tests/cache_control/data/user.json").unwrap();

  mock_server
    .mock("GET", "/users/1")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_header("Cache-Control", "public, max-age=600")
    .with_body(user_json)
    .create();

  mock_server
    .mock("GET", "/posts?id=1")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_header("Cache-Control", "public, max-age=300")
    .with_body(posts_json)
    .create();
}
