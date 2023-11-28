use std::collections::BTreeMap;

use crate::config::JoinType;

pub trait GraphQLOperationContext {
  fn selection_set(
    &self,
    type_subgraph_fields: Option<BTreeMap<String, (BTreeMap<String, Vec<(String, String)>>, Vec<JoinType>)>>,
    root_field_type: Option<String>,
    url: String,
    filter_selection_set: bool,
  ) -> Option<String>;
}
