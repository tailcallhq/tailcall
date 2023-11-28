use std::collections::HashMap;

use hyper::Uri;
use serde::de::{DeserializeOwned, Error};
use serde_json::{Map, Value};

use crate::blueprint::{is_scalar, Definition};
use crate::parser::de::{
  de_kebab, next_token, parse_args, parse_arguments_string, parse_operation, parse_selections, parse_selections_string,
  to_json, value_to_graphql_selections,
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
      let query = de_kebab(query);
      if let Ok(p) = serde_qs::from_str::<Map<String, Value>>(&query) {
        parser.selections = parse_selections(&p);
        parser.arguments = Some(parse_args(&p));
        return Ok(parser);
      }
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
    let input = match &self.selections {
      None => "",
      Some(s) => s,
    };
    let mut defi_hm = Value::Null;
    if input.contains('*') {
      defi_hm = build_definition_map(definations)?;
    }
    let hm = parse_selections_string(defi_hm, input, &self.operation)?;
    let v = Value::Object(hm);
    Ok(value_to_graphql_selections(&v))
  }
  fn parse_arguments(&mut self) -> Result<Value, serde_json::Error> {
    let matches = match &self.arguments {
      None => "",
      Some(s) => s,
    };
    let hm = parse_arguments_string(matches)?;
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
