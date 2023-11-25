use std::collections::HashMap;

use serde_json::Value;
type PosValHolder = HashMap<String, Vec<(String, String)>>;

pub fn de_kebab(qry: &str) -> String {
  let mut s = String::new();
  let mut b = false;
  for char in qry.chars() {
    match char {
      ' ' => (),
      '-' => {
        b = true;
      }
      _ => {
        if b {
          s.push(char.to_ascii_uppercase());
        } else {
          s.push(char);
        }
        b = false;
      }
    }
  }
  s
}

pub fn to_json(value: &Value, result: &mut HashMap<usize, PosValHolder>, prl: (Option<String>, &String, usize)) {
  match value {
    Value::Null | Value::Bool(_) | Value::Number(_) => (),
    Value::String(s) => {
      let (parent_key, root_node, level) = prl;
      let y = parent_key.unwrap_or_default();
      let v = result.entry(level).or_default().entry(root_node.clone()).or_default();
      if !y.is_empty() {
        v.push((y, s.clone()));
      }
    }
    Value::Array(arr) => {
      let (parent_key, root_node, level) = prl;
      for v in arr.iter() {
        to_json(v, result, (parent_key.clone(), root_node, level + 1));
      }
    }
    Value::Object(obj) => {
      let (_, root_node, level) = prl;
      for (k, v) in obj.iter() {
        to_json(
          v,
          result,
          (
            Some(k.to_string()),
            if v.is_object() { k } else { root_node },
            level + 1,
          ),
        );
      }
    }
  }
}

pub fn to_json_str(value: &Value) -> String {
  match value {
    Value::Null => "".to_string(), // Return empty string for null values
    Value::Bool(b) => b.to_string(),
    Value::Number(num) => num.to_string(),
    Value::String(s) => s.to_string(),
    Value::Array(arr) => {
      let elements: Vec<String> = arr.iter().map(to_json_str).collect();
      format!("[{}]", elements.join(" "))
    }
    Value::Object(obj) => {
      let pairs: Vec<String> = obj.iter().map(|(k, v)| get_cur(k, v)).collect();
      format!("{{{}}}", pairs.join(" "))
    }
  }
}

pub fn next_token(input: &str, position: &mut usize) -> Option<char> {
  if let Some(ch) = input.chars().nth(*position) {
    *position += 1;
    Some(ch)
  } else {
    None
  }
}

fn get_cur(k: &String, v: &Value) -> String {
  let s = to_json_str(v);
  if s.is_empty() {
    k.clone()
  } else {
    format!("{} {}", k, s)
  }
}
