use std::collections::BTreeMap;

use super::PathResolver;

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
