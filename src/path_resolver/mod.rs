use std::collections::BTreeMap;

use hyper::HeaderMap;
use indexmap::IndexMap;

use crate::json::JsonLike;
use crate::EnvIO;

/// Provides helper methods on top of existing values to very efficiently get a
/// value at a path.
pub trait PathResolver {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>;
}

impl PathResolver for async_graphql::Value {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        self.get_path(path).map(|v| v.to_owned())
    }
}

impl PathResolver for serde_json::Value {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        self.get_path(path)
            .and_then(|v| async_graphql::Value::from_json(v.to_owned()).ok())
    }
}

impl PathResolver for BTreeMap<String, String> {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        match path {
            [] => Some(async_graphql::Value::Object(indexmap::IndexMap::from_iter(
                self.iter()
                    .map(|(k, v)| (async_graphql::Name::new(k), async_graphql::Value::from(v))),
            ))),
            [key] => self.get(key.as_ref()).map(|v| v.into()),
            _ => None,
        }
    }
}

impl PathResolver for IndexMap<async_graphql::Name, async_graphql::Value> {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        if path.is_empty() {
            return Some(async_graphql::Value::Object(self.clone()));
        }

        self.get(path[0].as_ref())
            .and_then(|v| v.get_path_value(&path[1..]))
    }
}

impl<Env: EnvIO + ?Sized> PathResolver for Env {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        match &path {
            // TODO: no way to express all the envs with current trait, but do we even need this?
            [] => None,
            [key] => self.get(key.as_ref()).map(|v| v.into()),
            _ => None,
        }
    }
}

impl PathResolver for HeaderMap {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        match &path {
            // TODO: do we need ability to render all the headers? there could be security concerns
            // about this
            [] => None,
            [key] => self
                .get(key.as_ref())
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string().into()),
            _ => None,
        }
    }
}
