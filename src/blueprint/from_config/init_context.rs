use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use crate::config;
use crate::path::PathString;

#[derive(Debug)]
pub struct InitContext {
  pub vars: BTreeMap<String, String>,
  pub env_vars: HashMap<String, String>,
}

impl InitContext {
  pub fn env_var(&self, key: &str) -> Option<&str> {
    self.env_vars.get(key).map(|v| v.as_str())
  }

  pub fn var(&self, key: &str) -> Option<&str> {
    self.vars.get(key).map(|v| v.as_str())
  }
}

impl From<&config::Server> for InitContext {
  fn from(server: &config::Server) -> Self {
    Self { vars: server.vars.clone().0, env_vars: std::env::vars().collect() }
  }
}

impl PathString for InitContext {
  fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
    let ctx = self;

    if path.is_empty() {
      return None;
    }

    path.split_first().and_then(|(head, tail)| match head.as_ref() {
      "vars" => ctx.var(tail[0].as_ref()).map(|v| v.into()),
      "env" => ctx.env_var(tail[0].as_ref()).map(|v| v.into()),
      _ => None,
    })
  }
}