use std::collections::BTreeMap;

use crate::config::JoinType;

pub type FieldNameAndType = (String, String);
pub type FieldNameAndTypePairs = Vec<FieldNameAndType>;
pub type UrlToFieldNameAndTypePairsMap = BTreeMap<String, FieldNameAndTypePairs>;

pub trait GraphQLOperationContext {
  fn selection_set(
    &self,
    type_subgraph_fields: Option<BTreeMap<String, (UrlToFieldNameAndTypePairsMap, Vec<JoinType>)>>,
    field_type: Option<String>,
    url: String,
    filter_selection_set: bool,
  ) -> Option<String>;
}
