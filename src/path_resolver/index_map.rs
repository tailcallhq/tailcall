use indexmap::IndexMap;

use super::PathResolver;

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
