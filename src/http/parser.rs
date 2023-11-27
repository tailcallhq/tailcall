use std::collections::{HashMap, LinkedList};

use hyper::Uri;
use serde::de::{DeserializeOwned, Error};
use serde_json::{Map, Value};


use crate::blueprint::{is_scalar, Definition};
use crate::parser::de::{
  de_kebab, next_token, parse_args, parse_operation, parse_selections, to_json, value_to_graphql_selections,
};

#[derive(Debug, serde::Deserialize, Default)]
pub struct Parser {
  operation: String,
  arguments: Option<String>,
  selections: Option<String>,
}

impl Parser {
  pub fn from_path(uri: &Uri) -> anyhow::Result<Parser> {
    let mut parser = Self { operation: parse_operation(&de_kebab(uri.path())), ..Default::default() };

    if let Some(query) = uri.query() {
      let query = de_kebab(query).replace('\\', "");
      return match serde_qs::from_str::<Map<String, Value>>(&query) {
        Ok(p) => {
          parser.selections = parse_selections(&p);
          parser.arguments = Some(parse_args(&p)?);
          Ok(parser)
        }
        Err(e) => Err(anyhow::anyhow!("Unable to parse query: {e}")),
      };
    }
    Ok(parser)
  }
  pub fn parse<T: DeserializeOwned>(&mut self, definations: &Vec<Definition>) -> Result<T, serde_json::Error> {
    let s = self.parse_selections(definations)?;
    let v = self.parse_arguments()?;
    let v = self.parse_to_string(v, s)?;
    let mut hm = serde_json::Map::new();
    hm.insert("query".to_string(), Value::from(v));
    serde_json::from_value::<T>(Value::from(hm))
  }
  fn parse_selections(&mut self, definations: &Vec<Definition>) -> Result<String, serde_json::Error> {
    let mut hm = Map::new();
    let mut p = String::new();
    let mut curhm = &mut hm;
    let input = match &self.selections {
      None => "",
      Some(s) => s,
    };
    let mut defi_hm = Value::Null;
    if input.contains('*') {
      defi_hm = build_definition_map(definations)?;
    }
    let mut queue: LinkedList<String> = LinkedList::new();
    for char in input.chars() {
      match char {
        '*' => {
          let mut tmp_defi = &mut defi_hm;
          if let Some(first) = queue.front() {
            if !first.eq_ignore_ascii_case(&self.operation) {
              queue.push_front(self.operation.clone());
            }
          } else {
            queue.push_front(self.operation.clone());
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
    curhm.insert(p, Value::Null);
    let v = Value::Object(hm);
    Ok(value_to_graphql_selections(&v))
  }
  fn parse_arguments(&mut self) -> Result<Value, serde_json::Error> {
    let mut hm = Map::new();
    let mut p = String::new();
    let mut p1 = String::new();
    let mut curhm = &mut hm;
    let mut b = false;
    let matches = match &self.arguments {
      None => "",
      Some(s) => s,
    };
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
    Ok(Value::from(hm))
  }
  fn parse_to_string(&self, v: Value, sx: String) -> Result<String, serde_json::Error> {
    let mut hm = HashMap::new();
    to_json(&v, &mut hm, (None, &self.operation, 0));
    let mut s = if sx.eq("{}") {
      format!("{{{}}}", self.operation)
    } else {
      format!("{{{} {sx}}}", self.operation)
    };
    let mut pos = 0;
    let mut stk = 0usize;
    let mut p = String::new();
    while let Some(char) = next_token(&s, &mut pos) {
      match char {
        '{' => {
          stk += 1;
        }
        '}' => {
          if stk == 0 {
            return Err(serde_json::Error::custom("Unexpected token }"));
          }
          stk -= 1;
        }
        ' ' => {
          if let Some(x) = hm.get(&stk) {
            if let Some(v) = x.get(&p) {
              if !v.is_empty() {
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
          p = String::new();
        }
        _ => {
          p.push(char);
        }
      }
    }
    if stk > 0 {
      return Err(serde_json::Error::custom("Unexpected token {"));
    }
    Ok(s)
  }
}

fn build_definition_map(definitions: &Vec<Definition>) -> Result<Value, serde_json::Error> {
  let mut definition_map = Map::new();
  let mut current_definition = &mut definition_map;

  for definition in definitions {
    current_definition = current_definition
      .entry(definition.name())
      .or_insert_with(|| Value::Object(Map::new()))
      .as_object_mut()
      .unwrap();

    for field in definition.fields().unwrap_or(&vec![]) {
      if is_scalar(field.of_type.name()) {
        current_definition.insert(field.name.clone(), Value::Null);
      } else {
        current_definition.insert(field.name.clone(), Value::String(field.of_type.name().to_string()));
      }
    }

    current_definition = &mut definition_map;
  }

  create_query_map(&definition_map, definition_map.get("Query").unwrap().clone())
}

fn create_query_map(definition_map: &Map<String, Value>, query_value: Value) -> Result<Value, serde_json::Error> {
  let mut result_map = Map::new();

  for (key, val) in query_value.as_object().ok_or(serde_json::Error::custom(format!(
    "The field: {} is not an json",
    query_value
  )))? {
    match val {
      Value::String(subtype) => {
        let mapped_value = create_query_map(definition_map, definition_map.get(subtype).unwrap().clone());
        result_map.insert(key.clone(), mapped_value?);
      }
      _ => {
        result_map.insert(key.clone(), Value::Null);
      }
    }
  }

  Ok(Value::Object(result_map))
}
