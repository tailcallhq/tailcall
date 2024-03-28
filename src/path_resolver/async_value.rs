use super::PathResolver;
use crate::json::JsonLike;

impl PathResolver for async_graphql::Value {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        self.get_path(path).map(|v| v.to_owned())
    }
}
