use std::collections::BTreeMap;

pub type FieldNameAndType = (String, String);
pub type FieldNameAndTypePairs = Vec<FieldNameAndType>;
pub type UrlToFieldNameAndTypePairsMap = BTreeMap<String, FieldNameAndTypePairs>;
pub type ObjectNameToFieldPairsMap = BTreeMap<String, FieldNameAndTypePairs>;
pub type UrlToObjFieldsMap = BTreeMap<String, ObjectNameToFieldPairsMap>;

#[derive(Debug)]
pub struct SelectionSetFilterData {
  pub obj_name_to_fields_map: ObjectNameToFieldPairsMap,
  pub obj_name: String,
  pub url: String,
  pub url_obj_name_to_ids_map: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

pub trait GraphQLOperationContext {
  fn selection_set(
    &self,
    selection_set_filter: Option<SelectionSetFilterData>,
    filter_selection_set: bool,
  ) -> Option<String>;
}
