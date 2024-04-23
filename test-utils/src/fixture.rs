use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;

pub static ROOT_DIR: Lazy<PathBuf> = Lazy::new(|| project_root::get_project_root().unwrap());
pub static FIXTURES_DIR: Lazy<PathBuf> = Lazy::new(|| ROOT_DIR.join("tests/fixtures"));

pub fn get_fixture_path(path: impl AsRef<Path>) -> PathBuf {
    FIXTURES_DIR.join(path)
}
