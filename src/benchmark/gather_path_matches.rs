// src/benchmark/common.rs

use serde_json::Value;

pub fn gather_path_matches(input: &Value, path: &[&str]) -> Option<Value> {
  let mut current = input;
  for key in path {
    current = match current.get(key) {
      Some(value) => value,
      None => return None, // Handle the case where the key doesn't exist
    };
  }
  Some(current.clone())
}
