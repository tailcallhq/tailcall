use std::path::{Path, PathBuf};

pub fn get_fixture_path(path: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
