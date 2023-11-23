use crate::valid::Valid;

pub struct Operation {
  pub document: async_graphql::parser::types::ExecutableDocument,
}

impl Operation {
  fn from_gql(sdl: &str) -> Valid<Self, String> {
    match async_graphql::parser::parse_query(sdl) {
      Ok(doc) => Valid::succeed(Operation { document: doc }),
      Err(e) => Valid::fail(e.to_string()),
    }
  }

  pub fn from_file_path(file_path: &str) -> anyhow::Result<Operation> {
    Ok(Self::from_gql(&std::fs::read_to_string(file_path)?).to_result()?)
  }
}
