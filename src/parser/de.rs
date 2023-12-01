use std::collections::{HashMap, VecDeque};

use serde::de::Error;
use serde_json::{Map, Value};
type PosValHolder = HashMap<String, Vec<(String, String)>>;

pub fn parse_operation(mut path: &str) -> String {
  if !path.starts_with("/api/") {
    return String::new(); // empty string is treated as null
  }
  path = &path[5..];
  let path = de_kebab(path);
  let mut s = String::new();
  for char in path.chars() {
    if char.is_ascii_alphanumeric() {
      s.push(char);
    } else {
      break;
    }
  }
  s
}

pub fn parse_selections_string(
  mut defi_hm: Value,
  input: &str,
  operation: &str,
) -> Result<Map<String, Value>, serde_json::Error> {
  // let the root node (i.e. operation) be `foo`
  // so the query can be either in form of `foo.foo1.bar` or `foo.bar` or `bar`

  let mut p = String::new();
  let mut hm = Map::new();
  let mut curhm = &mut hm;
  let mut queue: VecDeque<String> = VecDeque::new();
  for char in input.chars() {
    match char {
      '*' => {
        parse_wildcard_selections(&mut curhm, &mut defi_hm, &mut queue, operation)?;
        curhm = &mut hm;
      }
      '.' => {
        if !p.is_empty() {
          curhm = curhm // as keys are seperated by `.` it creates nested object for nested key.
            .entry(&p)
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .unwrap();
          queue.push_back(p); // put the current key to the queue.
          p = String::new();
        }
      }
      ',' => {
        // queries are seperated by commas so the hashmap is set back to root and queue is cleared.
        queue.clear();
        curhm.insert(p, Value::Null);
        curhm = &mut hm;
        p = String::new();
      }
      _ => {
        // push the keys/chars to the string
        p.push(char);
      }
    }
  }
  curhm.insert(p, Value::Null);
  Ok(hm)
}
#[allow(clippy::too_many_arguments)]
#[inline]
fn parse_wildcard_selections(
  curhm: &mut &mut Map<String, Value>,
  mut tmp_defi: &mut Value,
  queue: &mut VecDeque<String>,
  operation: &str,
) -> Result<(), serde_json::Error> {
  if let Some(first) = queue.front() {
    if !first.eq_ignore_ascii_case(operation) {
      queue.push_front(operation.to_owned());
    }
  } else {
    queue.push_front(operation.to_owned()); // if queue does not contain root node then add it in the front.
  }
  let mut len = queue.len();
  loop {
    // this loop iterates over all the queried keys and gets till the last node requested.
    match len {
      1 => {
        let last = queue.pop_front().unwrap();
        **curhm = tmp_defi
          .get(&last)
          .ok_or(serde_json::Error::custom(format!("404 - key: {last} not found")))?
          .as_object()
          .cloned()
          .unwrap();
        break;
      }
      _ => {
        let key = queue.pop_front().unwrap();
        tmp_defi = tmp_defi
          .get_mut(&key)
          .ok_or(serde_json::Error::custom(format!("404 - key: {key} not found")))?;
      }
    }
    len -= 1;
  }
  Ok(())
}

pub fn parse_arguments_string(matches: &str) -> Result<Map<String, Value>, serde_json::Error> {
  let mut hm = Map::new();
  let mut p = String::new();
  let mut p1 = String::new();
  let mut curhm = &mut hm;
  let mut b = false;
  for char in matches.chars() {
    match char {
      '.' => {
        curhm = curhm
          .entry(p)
          .or_insert_with(|| Value::Object(Map::new()))
          .as_object_mut()
          .unwrap();
        p = String::new();
        b = false;
      }
      ',' => {
        b = false;
        curhm.insert(p, Value::from(p1));
        curhm = &mut hm;
        p = String::new();
        p1 = String::new();
      }
      '=' => {
        b = true;
      }
      _ => {
        if b {
          p1.push(char);
        } else {
          p.push(char);
        }
      }
    }
  }
  curhm.insert(p, Value::from(p1));
  Ok(hm)
}

pub fn parse_args(p: &Map<String, Value>) -> String {
  p.iter()
    .filter(|(key, _)| !key.starts_with("api") && !key.starts_with("/api") && !key.starts_with('$'))
    .map(|(k, v)| format!("{k}={}", v.as_str().unwrap_or_default()))
    .collect::<Vec<String>>()
    .join(",")
}

pub fn parse_selections(p: &serde_json::Map<String, Value>) -> Option<String> {
  Some(p.get("$")?.as_str()?.to_string())
}

pub fn de_kebab(qry: &str) -> String {
  let qry = urlencoding::decode(qry).unwrap_or_default().replace(['\\', ' '], "");
  let converter = convert_case::Converter::new();
  let converter = converter.to_case(convert_case::Case::Camel);
  converter.convert(qry)
}

pub fn deserialize_to_levelwise_args(
  value: &Value,
  result: &mut HashMap<usize, PosValHolder>,
  prl: (Option<String>, &String, usize),
) {
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
        deserialize_to_levelwise_args(v, result, (parent_key.clone(), root_node, level + 1));
      }
    }
    Value::Object(obj) => {
      let (_, root_node, level) = prl;
      for (k, v) in obj.iter() {
        deserialize_to_levelwise_args(
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

pub fn value_to_graphql_selections(value: &Value) -> String {
  match value {
    Value::Null => "".to_string(), // Return empty string for null values
    Value::Bool(b) => b.to_string(),
    Value::Number(num) => num.to_string(),
    Value::String(s) => s.to_string(),
    Value::Array(arr) => {
      let elements: Vec<String> = arr.iter().map(value_to_graphql_selections).collect();
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
  let s = value_to_graphql_selections(v);
  if s.is_empty() {
    k.clone()
  } else {
    format!("{} {}", k, s)
  }
}
