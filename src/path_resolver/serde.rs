use super::PathResolver;
use crate::json::JsonLike;

impl PathResolver for serde_json::Value {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        // FIXME: drop this implementation
        self.get_path(path)
            .and_then(|v| async_graphql::Value::from_json(v.to_owned()).ok())
    }
}
