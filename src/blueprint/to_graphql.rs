use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

fn read_file(path: &str) -> Result<String, std::io::Error> {
  let mut file = File::open(path)?;
  let mut src = String::new();
  file.read_to_string(&mut src)?;

  Ok(src)
}

fn load_pub_structs(src: &str) -> HashMap<String, String> {
  let mut structs = HashMap::new();
  let mut in_struct = false;

  let mut struct_name = String::new();

  for line in src.lines() {
    if line.starts_with("pub struct")  {
      in_struct = true;

      let mut parts = line.split_whitespace();
      struct_name = parts.nth(2).unwrap().to_string();

      structs.insert(struct_name.clone(), line.to_string());
    } else if in_struct {
      let mut r#struct = structs.get(&struct_name).unwrap().to_string();

      // it is possible to add a checker starts_with("pub") if needed to add only public fields
      r#struct.push_str("\n");
      r#struct.push_str(line);

      if line.ends_with("}") {
        in_struct = false;
      }

      structs.insert(struct_name.clone(), r#struct);
    }
  }

  structs
}

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
  fn loads_a_file() {
    let source_file = "src/blueprint/mod.rs";
    let result = read_file(source_file);
    assert!(result.is_ok());
  }

  #[test]
  fn finds_http_struct() {
    let source_file = "src/config/config.rs";
    let result = read_file(source_file);

    assert!(result.is_ok());

    let result = result.unwrap();

    let structs = load_pub_structs(result.as_str());

    assert!(structs.len() > 0);
    assert!(structs.contains_key("Http"));
  }
}
