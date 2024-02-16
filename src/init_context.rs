use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::blueprint;
use crate::{path::PathString, runtime::TargetRuntime};

pub struct InitContext {
    pub vars: BTreeMap<String, String>,
    pub runtime: TargetRuntime,
}

impl InitContext {
    pub fn new(server: &blueprint::Server, runtime: TargetRuntime) -> Self {
        Self { vars: server.vars.clone(), runtime }
    }

    pub fn env_var(&self, key: &str) -> Option<Cow<'_, str>> {
        self.runtime.env.get(key)
    }

    pub fn var(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|v| v.as_str())
    }
}

impl PathString for InitContext {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
        let ctx = self;

        if path.is_empty() {
            return None;
        }

        path.split_first()
            .and_then(|(head, tail)| match head.as_ref() {
                "vars" => ctx.var(tail[0].as_ref()).map(|v| v.into()),
                "env" => ctx.env_var(tail[0].as_ref()),
                _ => None,
            })
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub fn test_value() -> InitContext {
        InitContext {
            vars: Default::default(),
            runtime: crate::runtime::test::init(None),
        }
    }
}
