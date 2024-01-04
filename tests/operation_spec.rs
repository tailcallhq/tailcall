use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct OperationSpec {
  test_queries: Vec<OperationQuerySpec>,
}

#[derive(Debug)]
struct OperationQuerySpec {
  query: String,
  diagnostic_count: u8,
}

impl OperationSpec {
  fn new(_path: PathBuf, _content: &str) -> OperationSpec {
    // TODO: Fill this later
    OperationSpec { test_queries: vec![] }
  }

  fn cargo_read(path: &str) -> std::io::Result<Vec<OperationSpec>> {
    let mut dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir_path.push(path);

    let entries = fs::read_dir(dir_path.clone())?;
    let _files: Vec<OperationSpec> = vec![];

    for entry in entries {
      let path = entry?.path();
      if path.is_file() && path.extension().unwrap_or_default() == "graphql" {
        let contents = fs::read_to_string(path.clone())?;

        for component in contents.split("#>") {
          println!("{}", component);
        }
      }
    }
    Ok(vec![])
  }
}

#[test]
fn test_schema_operations() {
  let _specs = OperationSpec::cargo_read("tests/graphql/operations");
  println!("{:?}", _specs)
}
