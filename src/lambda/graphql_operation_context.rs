use std::collections::BTreeMap;

pub trait GraphQLOperationContext {
  fn selection_set(&self, type_subgraph_fields: Option<BTreeMap<String, BTreeMap<String, Vec<(String, String)>>>>, root_field_type: Option<String>, url: String) -> Option<String>;
}
