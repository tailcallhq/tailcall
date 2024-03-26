use hyper::HeaderMap;

use super::PathResolver;

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
