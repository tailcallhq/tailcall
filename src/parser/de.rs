use std::collections::{HashMap, LinkedList};

use serde::de::Error;
use serde_json::{Map, Value};
type PosValHolder = HashMap<String, Vec<(String, String)>>;

pub fn parse_operation(path: &str) -> String {
  if path.len() < 6 {
    return String::new();
  }
  let path = de_kebab(path);
  let mut s = String::new();
  for char in path.chars().skip(5) {
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
  let mut p = String::new();
  let mut hm = Map::new();
  let mut curhm = &mut hm;
  let mut queue: LinkedList<String> = LinkedList::new();
  for char in input.chars() {
    match char {
      '*' => {
        let mut tmp_defi = &mut defi_hm;
        if let Some(first) = queue.front() {
          if !first.eq_ignore_ascii_case(operation) {
            queue.push_front(operation.to_owned());
          }
        } else {
          queue.push_front(operation.to_owned());
        }
        let mut len = queue.len();
        while len > 0 {
          match len {
            1 => {
              let last = queue.pop_front().unwrap();
              *curhm = tmp_defi
                .get(&last)
                .ok_or(serde_json::Error::custom(format!("404 - key: {last} not found")))?
                .as_object()
                .cloned()
                .unwrap();
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
        curhm = &mut hm;
      }
      '.' => {
        if !p.is_empty() {
          let pc = p.clone();
          curhm = curhm
            .entry(&pc)
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .unwrap();
          queue.push_back(pc);
          p.clear();
        }
      }
      ',' => {
        queue.clear();
        curhm.insert(p.clone(), Value::Null);
        curhm = &mut hm;
        p.clear();
      }
      _ => {
        p.push(char);
      }
    }
  }
  curhm.insert(p.to_string(), Value::Null);
  Ok(hm)
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
          .entry(p.clone())
          .or_insert_with(|| Value::Object(Map::new()))
          .as_object_mut()
          .unwrap();
        p.clear();
        b = false;
      }
      ',' => {
        b = false;
        curhm.insert(p.clone(), Value::from(p1.clone()));
        curhm = &mut hm;
        p.clear();
        p1.clear();
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
