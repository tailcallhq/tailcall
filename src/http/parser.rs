use std::collections::HashMap;

use hyper::Uri;
use serde::de::{DeserializeOwned, Error};
use serde_json::{Map, Value};

use crate::blueprint::{is_scalar, Definition, FieldDefinition};
use crate::parser::de::{
  de_kebab, deserialize_to_levelwise_args, next_token, parse_args, parse_arguments_string, parse_operation,
  parse_selections, parse_selections_string, value_to_graphql_selections,
};

#[derive(Debug, serde::Deserialize, Default)]
pub struct Parser {
  operation: String,
  arguments: Option<String>,
  selections: Option<String>,
}

impl Parser {
  pub fn from_uri(uri: &Uri) -> anyhow::Result<Parser> {
    // Uri must have a path that contains operation (i.e. root node) as a route.
    // all the examples in comments will be given in reference to examples/jsonplaceholder.graphql
    let mut parser = Self { operation: parse_operation(&de_kebab(uri.path())), ..Default::default() };
    if let Some(query) = uri.query() {
      // it's optional to have query i.e. the selections and/or arguments.
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
    // let uri be: /api/posts\?\$=title&id=10
    let s = self.parse_selections(definations)?; // converts selections (i.e. query) to graphql request string
                                                 // this is what the string looks like: {posts {title}}

    let v = self.parse_arguments()?; // it will return serde_json::Value of arguments
                                     // for example, id=10 will be converted to {"id":"10"} to iterate easily

    let v = self.parse_to_string(v, s)?; // this functions is responsible to merge both of the values above.
                                         // this is what merged values look like: {posts (id: 10,) {title}}

    let mut hm = serde_json::Map::new();
    hm.insert("query".to_string(), Value::from(v)); // finally it puts it in the required form
    serde_json::from_value::<T>(Value::from(hm))
  }
  fn parse_selections(&mut self, definations: &Vec<Definition>) -> Result<String, serde_json::Error> {
    let input = match &self.selections {
      None => "",
      Some(s) => s,
    };
    let mut defi_hm = Value::Null;
    if input.contains('*') {
      defi_hm = build_definition_map(definations)?; // this iterates over all the nested Field definitions
                                                    // to convert it to serde_json::Value for easier parsing
                                                    // Basically it contains all the keys and nested values of the sdl.
    }
    let hm = parse_selections_string(defi_hm, input, &self.operation)?; // responsible to convert the query to graphql response string
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
    // let argument be id=10
    deserialize_to_levelwise_args(&v, &mut hm, (None, &self.operation, 0));
    // the function above fills the hashmap with level-wise argument for given key.
    // in the example, we need to put id=10 at level 1, so the hashmap will look like {1: {"posts": [("id", "10")]}}

    let mut s = if sx.eq("{}") {
      // the sting does not contain root (operation) so far.
      format!("{{{}}}", self.operation) //  if the query is empty it adds just the root node
    } else {
      format!("{{{} {sx}}}", self.operation) // else it adds current query string with root node
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
  // nothing fancy here, it just iterates over all the nested values and converts it to a map.
  let mut definition_map = Map::new();
  let mut current_definition = &mut definition_map;

  for definition in definitions {
    current_definition = current_definition
      .entry(definition.name())
      .or_insert_with(|| Value::Object(Map::new()))
      .as_object_mut()
      .unwrap();

    for field in get_fields(definition).unwrap_or(&vec![]) {
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

pub fn get_fields(definition: &Definition) -> Option<&Vec<FieldDefinition>> {
  match definition {
    Definition::ObjectTypeDefinition(f) => Some(&f.fields),
    _ => None,
  }
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
