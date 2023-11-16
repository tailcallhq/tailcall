use crate::config::Union;
use crate::blueprint::{UnionTypeDefinition, Definition};

pub fn to_union_type_definition((name, u): (&String, &Union)) -> Definition {
  Definition::UnionTypeDefinition(UnionTypeDefinition {
    name: name.to_owned(),
    description: u.doc.clone(),
    directives: Vec::new(),
    types: u.types.clone(),
  })
}