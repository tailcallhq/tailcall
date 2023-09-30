use std::fs::File;
use std::io::Read;

fn read_file(path: &str) -> Result<String, std::io::Error> {
  let mut file = File::open(path)?;
  let mut src = String::new();
  file.read_to_string(&mut src)?;

  Ok(src)
}

// Create the test structure
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fails_when_file_is_invalid() {
    let source_file = "some-unknown-location.rs";
    let result = read_file(source_file);
    assert!(result.is_err());
  }

  #[test]
  fn it_loads_a_file() {
    let source_file = "src/blueprint/mod.rs";
    let result = read_file(source_file);
    assert!(result.is_ok());
  }
}
