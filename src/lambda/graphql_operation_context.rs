use std::collections::BTreeMap;

use crate::config::JoinType;

pub type FieldNameAndType = (String, String);
pub type FieldNameAndTypePairs = Vec<FieldNameAndType>;
pub type UrlToFieldNameAndTypePairsMap = BTreeMap<String, FieldNameAndTypePairs>;

pub struct SelectionSetFilterData {
  pub type_subgraph_fields: BTreeMap<String, (UrlToFieldNameAndTypePairsMap, Vec<JoinType>)>,
  pub field_type: String,
  pub url: String,
}

pub trait GraphQLOperationContext {
  fn selection_set(
    &self,
    selection_set_filter: Option<SelectionSetFilterData>,
    filter_selection_set: bool,
  ) -> Option<String>;
}
