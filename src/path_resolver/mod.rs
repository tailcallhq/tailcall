mod async_value;
mod btree_map;
mod header_map;
mod index_map;
mod serde;

/// Provides helper methods on top of existing values to very efficiently get a
/// value at a path.
pub trait PathResolver {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>;
}
