use std::env::current_dir;
use std::path::{Path, PathBuf};

pub fn config_path(path: &Path) -> Result<PathBuf, std::io::Error> {
  let path = if path.is_relative() {
    let dir = current_dir()?;

    dir.join(path)
  } else {
    PathBuf::from(path)
  };

  Ok(path)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn relative_path() -> anyhow::Result<()> {
    let cwd = current_dir()?;

    assert_eq!(config_path(Path::new("tests/path.json"))?, cwd.join("tests/path.json"));
    assert_eq!(
      config_path(Path::new("./tests/path.json"))?,
      cwd.join("tests/path.json")
    );

    Ok(())
  }

  #[test]
  fn absolute_path() -> anyhow::Result<()> {
    assert_eq!(
      config_path(Path::new("/tests/path.json"))?,
      PathBuf::from(Path::new("/tests/path.json"))
    );

    Ok(())
  }
}
