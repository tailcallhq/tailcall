use crate::valid::Valid;

pub struct Operation {}

impl Operation {
  fn from_gql(sdl: &str) -> Valid<Self, String> {
    let doc = async_graphql::parser::parse_query(sdl);
    match doc {
      Ok(doc) => {
        println!("{:?}", doc);
        Valid::fail(format!("{:?}", doc))
      }
      Err(e) => Valid::fail(e.to_string()),
    }
  }

  pub fn from_file_path(file_path: &str) -> anyhow::Result<Operation> {
    Ok(Self::from_gql(&std::fs::read_to_string(file_path)?).to_result()?)
  }
}
