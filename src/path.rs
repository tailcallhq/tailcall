use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

lazy_static! {
  pub static ref RE: Regex = Regex::new(r"\{\{(?P<words>[\w\.]+)\}\}").unwrap();
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Path {
  pub segments: Vec<Segment>,
}
impl Path {
  pub fn new(segments: Vec<Segment>) -> Path {
    Path { segments }
  }
}
impl Display for Path {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut path = String::new();
    for segment in &self.segments {
      match segment {
        Segment::Literal { value } => {
          if !path.is_empty() {
            path.push('/');
          }
          path.push_str(value);
        }
        Segment::Param { location } => path.push_str(&format!("{{{:?}}}", location)),
      }
    }
    write!(f, "{}", path)
  }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Segment {
  Literal { value: String },
  Param { location: Vec<String> },
}
impl Segment {
  pub fn literal(literal: String) -> Segment {
    Segment::Literal { value: literal }
  }
  pub fn param(param: Vec<String>) -> Segment {
    Segment::Param { location: param }
  }
}

pub fn path_deserialize<'de, D>(deserializer: D) -> Result<Path, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;

  let segments: Result<Vec<_>, _> = s
    .split('/')
    .filter(|s| !s.is_empty())
    .map(|s| {
      if let Some(captures) = RE.captures(s) {
        captures.name("words").map_or_else(
          || Err(serde::de::Error::custom("invalid path")),
          |words| {
            let location: Vec<String> = words.as_str().split('.').map(String::from).collect();
            Ok(Segment::Param { location })
          },
        )
      } else {
        Ok(Segment::Literal { value: s.to_string() })
      }
    })
    .collect();

  segments.map(Path::new)
}

pub fn path_serialize<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let path_str: String = path
    .segments
    .iter()
    .map(|segment| match segment {
      Segment::Literal { value } => format!("/{}", value),
      Segment::Param { location } => format!("/{{{}}}", location.join(",")),
    })
    .collect();

  serializer.serialize_str(&path_str)
}
