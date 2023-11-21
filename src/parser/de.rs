use std::collections::HashMap;

use serde::de::{DeserializeOwned, Error};
use serde_json::{Map, Value};

use crate::async_graphql_hyper::GraphQLRequestLike;

type PosValHolder = HashMap<String, Vec<(String, String)>>;

#[derive(Debug)]
pub struct Parser {
  root: String,
  matches: String,
  input: String,
}

impl Parser {
  pub fn from_path(path: &str) -> Parser {
    let qry = path.split("api/").last().unwrap();
    let qry = de_kebab(qry);
    let mut root = String::new();
    let mut sel = String::new();
    let mut matches = String::new();
    let mut pr = String::new();
    let chars = qry.as_bytes();
    let mut i = 0;
    for _ in 0..chars.len() {
      let c = chars[i];
      i += 1;
      match c {
        b'?' => {
          break;
        }
        _ => {
          root.push(c as char);
        }
      }
    }
    let mut cur = 0usize;
    for c in chars.iter().skip(i) {
      match c {
        b'=' => {
          if pr.eq("$") {
            cur = 1;
            pr = String::new();
          } else {
            cur = 2;
            pr.push(*c as char);
          }
        }
        b'&' => {
          match cur {
            1 => {
              sel = pr.clone();
            }
            2 => {
              matches = pr.clone();
            }
            _ => {}
          }
          pr = String::new();
        }
        _ => {
          pr.push(*c as char);
        }
      }
    }
    match cur {
      1 => {
        sel = pr.clone();
      }
      2 => {
        matches = pr.clone();
      }
      _ => {}
    }
    Self { root, matches, input: sel }
  }
  pub fn parse<T: DeserializeOwned + GraphQLRequestLike>(&mut self) -> Result<T, serde_json::Error> {
    let s = self.parse_qry()?;
    let v = self.parse_matches()?;
    let v = self.parse_to_string(v, s)?;
    let mut hm = serde_json::Map::new();
    hm.insert("query".to_string(), Value::from(v));
    serde_json::from_value::<T>(Value::from(hm))
  }
  fn parse_qry(&mut self) -> Result<String, serde_json::Error> {
    let mut hm = Map::new();
    let mut p = String::new();
    let mut curhm = &mut hm;
    for c in self.input.chars() {
      match c {
        '.' => {
          if let Some(s) = curhm
            .entry(p.clone())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
          {
            curhm = s;
            p.clear();
          } else {
            return Err(serde_json::Error::custom("Error while parsing value"));
          }
        }
        ',' => {
          curhm.insert(p.clone(), Value::Null);
          curhm = &mut hm;
          p.clear();
        }
        _ => {
          p.push(c);
        }
      }
    }
    curhm.insert(p, Value::Null);
    let v = Value::Object(hm);
    Ok(to_json_str(&v))
  }
  fn parse_matches(&mut self) -> Result<Value, serde_json::Error> {
    let mut hm = Map::new();
    let mut p = String::new();
    let mut p1 = String::new();
    let mut curhm = &mut hm;
    let mut b = false;
    for c in self.matches.chars() {
      match c {
        '.' => {
          curhm = curhm
            .entry(p.clone())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("Expected Object");
          p.clear();
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
            p1.push(c);
          } else {
            p.push(c);
          }
        }
      }
    }
    curhm.insert(p, Value::from(p1));
    Ok(Value::from(hm))
  }
  fn parse_to_string(&self, v: Value, sx: String) -> Result<String, serde_json::Error> {
    let mut hm = HashMap::new();
    to_json(&v, &mut hm, (None, &self.root, 0));
    let mut s = format!("{{{} {sx}}}", self.root);
    let mut pos = 0;
    let mut stk = 0usize;
    let mut p = String::new();
    while let Some(c) = next_token(&s, &mut pos) {
      match c {
        '{' => {
          stk += 1;
        }
        '}' => {
          if stk == 0 {
            return Err(serde_json::Error::custom("Something went wrong while parsing"));
          }
          stk -= 1;
        }
        ' ' => {
          if let Some(x) = hm.get(&stk) {
            if let Some(v) = x.get(&p) {
              if !v.is_empty() {
                if v.first().unwrap().1.is_empty() {
                  break;
                }
                s.insert(pos, '(');
                pos += 1;
                for (k, v) in v {
                  let m = format!("{k}: {v},");
                  s.insert_str(pos, &m);
                  pos += m.len();
                  s.insert_str(pos, ") ");
                }
                pos += 2;
              }
            }
          }
          // hm.remove(&stk);
          p = String::new();
        }
        _ => {
          p.push(c);
        }
      }
    }
    Ok(s.clone())
  }
}

fn de_kebab(qry: &str) -> String {
  let mut s = String::new();
  let mut b = false;
  for c in qry.chars() {
    match c {
      '-' => {
        b = true;
      }
      _ => {
        if b {
          s.push(c.to_ascii_uppercase());
        } else {
          s.push(c);
        }
        b = false;
      }
    }
  }
  s
}

fn to_json(value: &Value, result: &mut HashMap<usize, PosValHolder>, prl: (Option<String>, &String, usize)) {
  match value {
    Value::Null | Value::Bool(_) | Value::Number(_) => (),
    Value::String(s) => {
      let (parent_key, root_node, level) = prl;
      let y = parent_key.unwrap_or_default();
      result
        .entry(level)
        .or_default()
        .entry(root_node.clone())
        .or_default()
        .push((y, s.clone()));
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

fn to_json_str(value: &Value) -> String {
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

fn next_token(input: &str, position: &mut usize) -> Option<char> {
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

#[cfg(test)]
mod de_tests {
  use crate::async_graphql_hyper::GraphQLRequest;
  use crate::parser::de::Parser;

  #[test]
  fn parse_t() {
    let mut parser = Parser::from_path("https://example.com/api/user?id=123&$=name,age,address.city,address.state");
    let x = parser.parse::<GraphQLRequest>();
    println!("{:?}", x);
  }
}
