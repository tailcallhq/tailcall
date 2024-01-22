use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::path::PathString;
use crate::{blueprint, EnvIO, HttpIO};

#[derive(Debug, Default)]
pub struct InitContext<Http, Env> {
  pub vars: BTreeMap<String, String>,
  pub env: Arc<Env>,
  pub http_client: Arc<Http>,
}

impl<Http: HttpIO, Env: EnvIO> InitContext<Http, Env> {
  pub fn new(server: &blueprint::Server, env: Arc<Env>, http_client: Arc<Http>) -> Self {
    Self { vars: server.vars.clone(), env, http_client }
  }

  pub fn env_var(&self, key: &str) -> Option<Cow<'_, str>> {
    self.env.get(key)
  }

  pub fn var(&self, key: &str) -> Option<&str> {
    self.vars.get(key).map(|v| v.as_str())
  }
}

impl<Http: HttpIO, Env: EnvIO> PathString for InitContext<Http, Env> {
  fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
    let ctx = self;

    if path.is_empty() {
      return None;
    }

    path.split_first().and_then(|(head, tail)| match head.as_ref() {
      "vars" => ctx.var(tail[0].as_ref()).map(|v| v.into()),
      "env" => ctx.env_var(tail[0].as_ref()),
      _ => None,
    })
  }
}
