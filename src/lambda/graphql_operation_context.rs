use std::collections::BTreeMap;

pub type FieldNameAndType = (String, String);
pub type FieldNameAndTypePairs = Vec<FieldNameAndType>;
pub type UrlToFieldNameAndTypePairsMap = BTreeMap<String, FieldNameAndTypePairs>;
pub type ObjectNameToFieldPairsMap = BTreeMap<String, FieldNameAndTypePairs>;
pub type UrlToObjFieldsMap = BTreeMap<String, ObjectNameToFieldPairsMap>;

#[derive(Debug)]
pub struct SelectionSetFilterData {
  pub url_obj_fields: UrlToObjFieldsMap,
  pub obj_name: String,
  pub url: String,
  pub url_obj_ids: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

pub trait GraphQLOperationContext {
  fn selection_set(
    &self,
    selection_set_filter: Option<SelectionSetFilterData>,
    filter_selection_set: bool,
  ) -> Option<String>;
}
